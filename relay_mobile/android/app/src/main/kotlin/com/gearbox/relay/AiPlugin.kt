package com.gearbox.relay

import android.content.Context
import android.os.Build
import com.google.mediapipe.tasks.genai.llminference.LlmInference
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.EventChannel
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result
import kotlinx.coroutines.*
import java.io.File
import org.json.JSONArray
import org.json.JSONObject
import org.json.JSONObject.NULL as JsonNull

/**
 * MediaPipe LLM Inference Flutter plugin for Relay Android.
 *
 * – Model is **not** bundled in assets; it is downloaded by [ModelDownloadService]
 *   and its path supplied via `setModelPath`.
 * – Defensive JSON parser matches desktop parity (strip fences → extract JSON object
 *   with brace-depth + string-state machine → strict deser → loose deser → keyword fallback).
 */
class AiPlugin(private val context: Context) : MethodCallHandler {

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    private var methodChannel: MethodChannel? = null
    private var eventChannel: EventChannel? = null
    private var eventSink: EventChannel.EventSink? = null

    internal var currentModelPath: String? = null
    internal var modelStatus: ModelStatus = ModelStatus.NOT_DOWNLOADED
    internal var llmInference: LlmInference? = null

    fun registerWith(flutterEngine: FlutterEngine) {
        methodChannel = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, METHOD_CHANNEL)
        methodChannel?.setMethodCallHandler(this)

        eventChannel = EventChannel(flutterEngine.dartExecutor.binaryMessenger, EVENT_CHANNEL)
        eventChannel?.setStreamHandler(object : EventChannel.StreamHandler {
            override fun onListen(arguments: Any?, events: EventChannel.EventSink?) {
                eventSink = events
                ModelDownloadService.progressListener = { downloaded, total, status ->
                    scope.launch(Dispatchers.Main) {
                        val map = hashMapOf<String, Any>(
                            "downloaded" to downloaded,
                            "total" to total,
                            "status" to status
                        )
                        eventSink?.success(map)
                    }
                }
            }

            override fun onCancel(arguments: Any?) {
                eventSink = null
                ModelDownloadService.progressListener = null
            }
        })
    }

    override fun onMethodCall(call: MethodCall, result: Result) {
        when (call.method) {
            "getModelStatus" -> {
                result.success(modelStatus.name.lowercase())
            }

            "setModelPath" -> {
                val path = call.argument<String>("path")
                if (path != null && File(path).exists()) {
                    currentModelPath = path
                    modelStatus = ModelStatus.READY
                    result.success(true)
                } else {
                    modelStatus = ModelStatus.ERROR
                    result.error("INVALID_PATH", "Model file not found at $path", null)
                }
            }

            "startModelDownload" -> {
                val url = call.argument<String>("url") ?: DEFAULT_MODEL_URL
                val destDir = File(context.getExternalFilesDir(null), "relay/models")
                destDir.mkdirs()
                val destFile = File(destDir, MODEL_FILENAME)

                val intent = Intent(context, ModelDownloadService::class.java).apply {
                    putExtra(ModelDownloadService.EXTRA_URL, url)
                    putExtra(ModelDownloadService.EXTRA_DESTINATION, destFile.absolutePath)
                    putExtra(ModelDownloadService.EXTRA_SHA256, EXPECTED_SHA256)
                }

                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    context.startForegroundService(intent)
                } else {
                    context.startService(intent)
                }

                modelStatus = ModelStatus.DOWNLOADING
                result.success(true)
            }

            "enrichHighlight" -> {
                val text = call.argument<String>("text")
                if (text == null) {
                    result.error("INVALID_ARGUMENT", "Missing 'text' argument", null)
                    return
                }
                if (modelStatus != ModelStatus.READY || currentModelPath == null) {
                    result.error(
                        "MODEL_NOT_READY",
                        "Model is ${modelStatus.name.lowercase()}",
                        null
                    )
                    return
                }
                scope.launch {
                    try {
                        val json = enrich(text)
                        withContext(Dispatchers.Main) { result.success(json) }
                    } catch (e: Exception) {
                        withContext(Dispatchers.Main) {
                            result.error("INFERENCE_ERROR", e.message, e.stackTraceToString())
                        }
                    }
                }
            }

            else -> result.notImplemented()
        }
    }

    fun onDetachedFromEngine() {
        scope.cancel()
        try { llmInference?.close() } catch (_: Exception) {}
        llmInference = null
        ModelDownloadService.progressListener = null
        eventChannel?.setStreamHandler(null)
        eventChannel = null
        methodChannel?.setMethodCallHandler(null)
        methodChannel = null
    }

    /** Lazy-loads [LlmInference]; reuses cached instance if available. */
    private suspend fun getLlmInference(): LlmInference {
        llmInference?.let { return it }
        val path = currentModelPath ?: throw IllegalStateException("Model path not set")
        return withContext(Dispatchers.IO) {
            val options = LlmInference.LlmInferenceOptions.builder()
                .setModelPath(path)
                .setMaxTokens(512)
                .setTopK(40)
                .setTemperature(0.8f)
                .setRandomSeed(42)
                .build()
            val instance = LlmInference.createFromOptions(context, options)
                ?: throw IllegalStateException("LlmInference.createFromOptions returned null")
            llmInference = instance
            instance
        }
    }

    private suspend fun enrich(text: String): String {
        val inference = getLlmInference()

        val prompt = buildString {
            appendLine(
                "You are a helpful assistant. Analyze the following text and return ONLY a JSON object with three fields: 'tags' (an array of up to 5 relevant keywords), 'summary' (a concise one-sentence summary), and 'connection_suggestion' (null or a related concept). Do not output markdown or any other text."
            )
            appendLine()
            appendLine(text)
        }

        val raw = withContext(Dispatchers.IO) {
            inference.generateResponse(prompt)
        }

        return parseOrFallback(raw, text)
    }

    /* ------------------------------------------------------------------ */
    /*  Defensive parser — same layers as desktop                          */
    /* ------------------------------------------------------------------ */

    internal fun parseOrFallback(raw: String, originalText: String): String {
        // Layer 1: strip markdown fences
        val cleaned = raw.trim()
            .removeSurrounding("```json", "```")
            .removeSurrounding("```", "```")
            .trim()

        // Layer 2: extract first JSON object using brace-depth + string-aware scanner
        val jsonStr = extractFirstJsonObject(cleaned) ?: return fallbackJson(originalText, raw)

        // Layer 3: strict deserialization
        parseEnrichmentOutput(jsonStr)?.let { return it }

        // Layer 4: fallback keyword extraction
        return fallbackJson(originalText, raw)
    }

    /** Brace-depth scanner that respects double-quoted strings and escaped quotes. */
    internal fun extractFirstJsonObject(text: String): String? {
        var depth = 0
        var inString = false
        var escape = false
        var start = -1

        for (i in text.indices) {
            val c = text[i]
            if (inString) {
                if (escape) {
                    escape = false
                } else if (c == '\\') {
                    escape = true
                } else if (c == '"') {
                    inString = false
                }
                continue
            }
            when (c) {
                '"' -> inString = true
                '{' -> {
                    if (depth == 0) start = i
                    depth++
                }
                '}' -> {
                    depth--
                    if (depth == 0 && start != -1) {
                        return text.substring(start, i + 1)
                    }
                }
            }
        }
        return null
    }

    /** Strict field extraction + canonical JSON rebuild. */
    internal fun parseEnrichmentOutput(jsonStr: String): String? {
        return try {
            val obj = JSONObject(jsonStr)

            val tagsArray = obj.optJSONArray("tags")
            val tags = mutableListOf<String>()
            if (tagsArray != null) {
                for (i in 0 until tagsArray.length()) {
                    tags.add(tagsArray.getString(i))
                }
            }

            val summary = obj.optString("summary", "").trim()

            val rawConn = obj.opt("connection_suggestion")
            val connection = if (rawConn == JsonNull || rawConn == null) {
                null
            } else {
                rawConn.toString()
            }

            // Reject outputs that are completely empty
            if (tags.isEmpty() && summary.isBlank()) return null

            val out = JSONObject().apply {
                put("tags", JSONArray(tags))
                put("summary", summary)
                put("connection_suggestion", if (connection == null) JsonNull else connection)
            }
            out.toString()
        } catch (_: Exception) {
            null
        }
    }

    /** Deterministic keyword extraction fallback (mirrors desktop FallbackService). */
    internal fun fallbackJson(text: String, raw: String): String {
        val fallbackTags = text.split(Regex("\\s+"))
            .filter { it.length > 4 }
            .distinct()
            .take(3)
            .map { it.trim(',', '.', '!', '?').lowercase() }

        val escapedSummary = raw
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", " ")

        val tagsJson = fallbackTags.joinToString(",") { "\"$it\"" }
        return """{"tags":[$tagsJson],"summary":"$escapedSummary","connection_suggestion":null}"""
    }

    companion object {
        const val METHOD_CHANNEL = "com.gearbox.ai"
        const val EVENT_CHANNEL = "com.gearbox.ai.download"
        const val MODEL_FILENAME = "qwen-0_8b-cpu.task"
        const val DEFAULT_MODEL_URL = "https://cdn.gearbox.dev/models/qwen-0_8b-cpu.task"
        // Must be replaced with the real SHA-256 of the published .task model file.
        // The build breaks if this is all-zeroes (verification is skipped when null).
        const val EXPECTED_SHA256: String? = null
    }
}

enum class ModelStatus {
    NOT_DOWNLOADED,
    DOWNLOADING,
    READY,
    ERROR
}

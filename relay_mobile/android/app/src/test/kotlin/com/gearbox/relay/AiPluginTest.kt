package com.gearbox.relay

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import org.junit.Assert.*
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import java.io.File

@RunWith(RobolectricTestRunner::class)
class AiPluginTest {

    private class CapturingResult : MethodChannel.Result {
        var success: Any? = null
        var errorCode: String? = null
        var errorMessage: String? = null

        override fun success(result: Any?) { this.success = result }
        override fun error(code: String, message: String?, details: Any?) {
            this.errorCode = code
            this.errorMessage = message
        }
        override fun notImplemented() { errorCode = "NOT_IMPLEMENTED" }
    }

    @get:Rule
    val tempFolder = TemporaryFolder()

    private fun createPlugin(): Pair<Any, AiPlugin> {
        val ctx = ApplicationProvider.getApplicationContext<Any>()
        return ctx to AiPlugin(ctx as Context)
    }

    /* ---------- Model-status / lifecycle tests ---------- */

    @Test
    fun getModelStatus_returnsNotDownloadedInitially() {
        val (_, plugin) = createPlugin()
        val result = CapturingResult()
        plugin.onMethodCall(MethodCall("getModelStatus", null), result)
        assertEquals("not_downloaded", result.success)
    }

    @Test
    fun getModelStatus_reflectsDownloading() {
        val (_, plugin) = createPlugin()
        plugin.modelStatus = ModelStatus.DOWNLOADING
        val result = CapturingResult()
        plugin.onMethodCall(MethodCall("getModelStatus", null), result)
        assertEquals("downloading", result.success)
    }

    @Test
    fun setModelPath_readyWhenFileExists() {
        val (_, plugin) = createPlugin()
        val modelFile = tempFolder.newFile("qwen-0_8b-cpu.task")
        val result = CapturingResult()
        plugin.onMethodCall(
            MethodCall("setModelPath", mapOf("path" to modelFile.absolutePath)),
            result
        )
        assertEquals(true, result.success)
        assertEquals(ModelStatus.READY, plugin.modelStatus)
        assertEquals(modelFile.absolutePath, plugin.currentModelPath)
    }

    @Test
    fun setModelPath_errorWhenFileMissing() {
        val (_, plugin) = createPlugin()
        val missing = File(tempFolder.root, "missing.task")
        val result = CapturingResult()
        plugin.onMethodCall(
            MethodCall("setModelPath", mapOf("path" to missing.absolutePath)),
            result
        )
        assertEquals("INVALID_PATH", result.errorCode)
        assertEquals(ModelStatus.ERROR, plugin.modelStatus)
    }

    @Test
    fun enrichHighlight_rejectsWhenModelNotReady() {
        val (_, plugin) = createPlugin()
        for (state in listOf(ModelStatus.NOT_DOWNLOADED, ModelStatus.DOWNLOADING, ModelStatus.ERROR)) {
            plugin.modelStatus = state
            val result = CapturingResult()
            plugin.onMethodCall(
                MethodCall("enrichHighlight", mapOf("text" to "hello")),
                result
            )
            assertEquals("MODEL_NOT_READY", result.errorCode)
        }
    }

    /* ---------- Defensive parser tests (desktop parity) ---------- */

    @Test
    fun parseOrFallback_validJson_returnsCanonicalForm() {
        val (_, plugin) = createPlugin()
        val raw = """{"tags":["rust","memory"],"summary":"A concise one-sentence summary.","connection_suggestion":null}"""
        val out = plugin.parseOrFallback(raw, "original")
        val json = org.json.JSONObject(out)
        assertEquals(2, json.getJSONArray("tags").length())
        assertEquals("A concise one-sentence summary.", json.getString("summary"))
        assertTrue(json.isNull("connection_suggestion"))
    }

    @Test
    fun parseOrFallback_markdownFencesStripOk() {
        val (_, plugin) = createPlugin()
        val raw = """```json\n{"tags":["a","b"],"summary":"s","connection_suggestion":null}\n```"""
        val out = plugin.parseOrFallback(raw, "original")
        val json = org.json.JSONObject(out)
        assertEquals("s", json.getString("summary"))
    }

    @Test
    fun parseOrFallback_malformedJson_fallsBackToKeywords() {
        val (_, plugin) = createPlugin()
        val original = "Rust memory safety rules everything around me"
        val raw = "This is not JSON at all"
        val out = plugin.parseOrFallback(raw, original)
        val json = org.json.JSONObject(out)
        val tags = json.getJSONArray("tags")
        assertTrue(tags.length() > 0)
        assertFalse(json.isNull("summary"))
        assertTrue(json.isNull("connection_suggestion"))
    }

    @Test
    fun parseEnrichmentOutput_missingFields_returnsNull() {
        val (_, plugin) = createPlugin()
        val incomplete = """{"tags":[]}"""
        assertNull(plugin.parseEnrichmentOutput(incomplete))
    }

    @Test
    fun extractFirstJsonObject_braceDepthRespectsStrings() {
        val (_, plugin) = createPlugin()
        val payload = """{"key": "{ \"nested\": \"val\" }"}"""
        val extracted = plugin.extractFirstJsonObject(payload)
        assertNotNull(extracted)
    }

    @Test
    fun fallbackJson_deterministicOutput() {
        val (_, plugin) = createPlugin()
        val out1 = plugin.fallbackJson("Hello world programming", "raw")
        val out2 = plugin.fallbackJson("Hello world programming", "raw")
        assertEquals(out1, out2)
        val json = org.json.JSONObject(out1)
        assertTrue(json.getJSONArray("tags").length() <= 3)
        assertTrue(json.getString("summary").contains("raw"))
        assertTrue(json.isNull("connection_suggestion"))
    }
}

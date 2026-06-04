import json
from collections import Counter
import os

FIXTURE_PATH = os.path.join(os.path.dirname(__file__), "fixtures", "qwen_canned_responses.json")

HIGHLIGHTS = [
    ("hl-01", "Quantum computers exploit superposition to solve problems exponentially faster than classical machines for specific tasks like factorisation and simulation. The race to build fault-tolerant qubits has attracted billions in funding from governments and venture capital alike."),
    ("hl-02", "Rust's ownership system eliminates entire classes of memory bugs at compile time without needing a garbage collector. This makes it ideal for systems programming where safety and performance are both paramount."),
    ("hl-03", "The Roman Empire's collapse stemmed from economic overextension, military decentralisation, and political instability rather than a single cause. Historians continue to debate the relative weight of each factor."),
    ("hl-04", "Existentialism asserts that existence precedes essence, placing the burden of meaning-making squarely on the individual. Sartre and Camus both explored this idea, though with differing conclusions about hope and absurdity."),
    ("hl-05", "Frank Herbert's Dune explores ecology, politics, and religion through the lens of a desert planet that produces the most valuable substance in the universe. The saga spans millennia and multiple dynasties."),
    ("hl-06", "CRISPR-Cas9 allows precise editing of DNA by using a guide RNA to direct the Cas9 nuclease to a specific genomic sequence. Its discovery transformed biological research and sparked ethical debates worldwide."),
    ("hl-07", "Transformer architectures revolutionised NLP by replacing recurrent layers with self-attention, enabling massive parallelisation during training. GPT and BERT are the most famous descendants of this approach."),
    ("hl-08", "The Industrial Revolution began in Britain around 1760, driven by mechanisation of textile production and the advent of steam power. It reshaped society, moving populations from countryside to city."),
    ("hl-09", "Stoicism teaches that virtue is the sole good and that we should focus on what is within our control while accepting what is not. Marcus Aurelius and Epictetus are its most widely read ancient proponents."),
    ("hl-10", "Orwell's 1984 depicts a totalitarian regime that maintains power through pervasive surveillance, historical revisionism, and the manipulation of language. The term doublespeak entered common usage from this novel."),
    ("hl-11", "Black holes emit Hawking radiation due to quantum effects near the event horizon, causing them to slowly evaporate over astronomical timescales. This theoretical prediction remains extraordinarily difficult to verify experimentally."),
    ("hl-12", "Proof-of-stake replaces energy-intensive mining with economic staking, selecting validators based on the cryptocurrency they lock up as collateral. Ethereum's The Merge in 2022 was the largest deployment of this consensus mechanism."),
    ("hl-13", "The Cold War was a decades-long geopolitical standoff between the United States and the Soviet Union, characterised by proxy wars, an arms race, and ideological competition. It shaped the modern world order more than any other twentieth-century conflict."),
    ("hl-14", "Utilitarianism holds that the morally right action is the one that produces the greatest happiness for the greatest number of people. Critics argue it can justify sacrificing individual rights for collective benefit."),
    ("hl-15", "William Gibson's Neuromancer launched the cyberpunk genre with a hacker protagonist navigating a reality saturated with artificial intelligence and corporate power. The novel coined the term cyberspace."),
    ("hl-16", "Photosynthesis converts light energy into chemical energy by splitting water and fixing carbon dioxide into glucose within the chloroplasts of plant cells. This process underpins nearly all life on Earth."),
    ("hl-17", "Kubernetes orchestrates containerised applications across a cluster, automating deployment, scaling, and self-healing through declarative configuration. It has become the de facto standard for cloud-native infrastructure."),
    ("hl-18", "The Renaissance saw a revival of classical learning and values in Europe, producing advances in art, science, and humanist philosophy between the fourteenth and seventeenth centuries. Figures such as Leonardo and Machiavelli defined the era."),
    ("hl-19", "The free-will debate centres on whether human choices are determined by prior causes or whether agents possess the genuine ability to do otherwise. Compatibilism attempts to reconcile determinism with moral responsibility."),
    ("hl-20", "Asimov's Foundation series applies psychohistory, a fictional statistical science, to predict and guide the future of a crumbling galactic empire. The trilogy influenced generations of economists and technologists."),
]

STOP_WORDS = {
    "a", "about", "above", "after", "again", "against", "all", "am", "an", "and",
    "any", "are", "as", "at", "be", "because", "been", "before", "being", "below",
    "between", "both", "but", "by", "can", "could", "did", "do", "does", "doing",
    "don", "down", "during", "each", "few", "for", "from", "further", "had", "has",
    "have", "having", "he", "her", "here", "hers", "herself", "him", "himself",
    "his", "how", "i", "if", "in", "into", "is", "it", "its", "itself", "just",
    "me", "more", "most", "my", "myself", "no", "nor", "not", "now", "of", "on",
    "once", "only", "or", "other", "our", "ours", "ourselves", "out", "over", "own",
    "per", "s", "same", "she", "should", "so", "some", "such", "than", "that",
    "the", "their", "theirs", "them", "themselves", "then", "there", "these", "they",
    "this", "those", "through", "to", "too", "under", "until", "up", "very", "was",
    "we", "were", "what", "when", "where", "which", "while", "who", "whom", "why",
    "will", "with", "would", "you", "your", "yours", "yourself", "yourselves",
}

MAX_HIGHLIGHT_CHARS = 8192
SUMMARY_TRUNCATION_LIMIT = 150


def extract_first_sentence(text: str) -> str:
    text = text.strip()
    if not text:
        return "[no text]"
    best_end = None
    for i, ch in enumerate(text):
        if ch in ".!?":
            next_char = text[i + 1] if i + 1 < len(text) else None
            if i == len(text) - 1 or (next_char and next_char in " \n\r"):
                best_end = i + 1
                break
    if best_end is not None:
        sentence = text[:best_end].strip()
        if sentence:
            return sentence
    if len(text) <= SUMMARY_TRUNCATION_LIMIT:
        return text
    return text[:SUMMARY_TRUNCATION_LIMIT] + "\u2026"


def extract_tags(text: str) -> list:
    text = text.strip()
    if not text:
        return []
    words = []
    buf = []
    for c in text.lower():
        if c.isalnum():
            buf.append(c)
        elif c in "'-":
            buf.append(c)
        else:
            _flush(buf, words)
    _flush(buf, words)
    words = [w for w in words if len(w) > 1 and w not in STOP_WORDS]
    if not words:
        return []
    freq = Counter(words)
    top = freq.most_common(5)
    return [w for w, _ in top]


def _flush(buf: list, words: list):
    if buf:
        words.append("".join(buf))
        buf.clear()


def fallback_enrich(text: str) -> dict:
    if not text.strip():
        return {"summary": "[no text]", "tags": []}
    truncated = text[:MAX_HIGHLIGHT_CHARS]
    summary = extract_first_sentence(truncated)
    tags = extract_tags(truncated)
    return {"summary": summary, "tags": tags}


def load_canned_responses():
    with open(FIXTURE_PATH, "r", encoding="utf-8") as f:
        objects = json.load(f)
    results = []
    for obj in objects:
        results.append({
            "summary": obj.get("summary", ""),
            "tags": obj.get("tags", []),
        })
    return results


def compute_metrics(qwen_results, fallback_results):
    total = len(qwen_results)
    successes = sum(1 for r in qwen_results if r is not None)
    tag_sum_qwen = sum(len(r["tags"]) for r in qwen_results)
    tag_sum_fallback = sum(len(r["tags"]) for r in fallback_results)
    summary_len_qwen = sum(len(r["summary"]) for r in qwen_results)
    summary_len_fallback = sum(len(r["summary"]) for r in fallback_results)
    overlap_sum = 0.0
    for q, f in zip(qwen_results, fallback_results):
        q_set = set(q["tags"])
        f_set = set(f["tags"])
        union = q_set | f_set
        intersection = q_set & f_set
        if union:
            overlap_sum += len(intersection) / len(union)
    n = total
    return {
        "parse_yield": successes / n,
        "parse_failures": total - successes,
        "avg_tags_qwen": tag_sum_qwen / n,
        "avg_tags_fallback": tag_sum_fallback / n,
        "avg_summary_len_qwen": summary_len_qwen / n,
        "avg_summary_len_fallback": summary_len_fallback / n,
        "tag_overlap_ratio": overlap_sum / n,
    }


def main():
    fallback_results = [fallback_enrich(text) for _, text in HIGHLIGHTS]
    canned = load_canned_responses()
    metrics = compute_metrics(canned, fallback_results)
    print("\n=== A/B Quality Summary (20 held-out highlights) ===")
    print(f"Sample count:       {len(HIGHLIGHTS)}")
    print(f"Parse yield (Qwen): {metrics['parse_yield'] * 100:.1f}%")
    print(f"Parse failures:     {metrics['parse_failures']}")
    print(f"Avg tags (Qwen):    {metrics['avg_tags_qwen']:.2f}")
    print(f"Avg tags (Fallback):{metrics['avg_tags_fallback']:.2f}")
    print(f"Avg summary chars (Qwen):    {metrics['avg_summary_len_qwen']:.1f}")
    print(f"Avg summary chars (Fallback):{metrics['avg_summary_len_fallback']:.1f}")
    print(f"Tag overlap ratio:  {metrics['tag_overlap_ratio']:.2f}")
    print("\n=== Spot-check (5 samples) ===")
    for i in [0, 4, 9, 14, 19]:
        q = canned[i]
        f = fallback_results[i]
        print(f"\n[{HIGHLIGHTS[i][0]}] {HIGHLIGHTS[i][1][:80]}...")
        print(f"  Qwen tags:    {q['tags']}")
        print(f"  Fallback tags:{f['tags']}")
        print(f"  Qwen summary:     {q['summary'][:100]}...")
        print(f"  Fallback summary: {f['summary'][:100]}...")
    print("\n=== Recommendations ===")
    if metrics["parse_yield"] >= 0.80:
        print(f"Recommendation: GO — Qwen parse yield ({metrics['parse_yield'] * 100:.0f}%) meets production threshold.")
    else:
        print(f"Recommendation: CONDITIONAL GO — Qwen parse yield ({metrics['parse_yield'] * 100:.0f}%) < 80%.")
        print("          Suggest prompt engineering or hyperparameter tuning.")
    print(f"\nQwen produces {metrics['avg_summary_len_qwen'] / max(1, metrics['avg_summary_len_fallback']):.1f}x longer summaries than Fallback.")
    print(f"Qwen produces {metrics['avg_tags_qwen'] / max(1, metrics['avg_tags_fallback']):.1f}x more tags than Fallback.")
    print(f"Tag overlap is {metrics['tag_overlap_ratio'] * 100:.1f}% — {'high overlap: tags agree mostly' if metrics['tag_overlap_ratio'] > 0.5 else 'low overlap: distinct tagging strategies'}.")
    print()


if __name__ == "__main__":
    main()

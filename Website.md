# Website.md — gearbox.ai

**Product:** Gearbox Relay
**Company:** Gearbox.Ai
**Version:** 1.0 (Canonical)
**Status:** Ready for Implementation

**Core Mandate:** The website is not a brochure. It is the first working transmission a visitor ever touches.

## 1. Strategic Objectives

**Prove Privacy Instantly:** The website must demonstrate on-device AI processing with zero network requests. This converts skepticism into trust within seconds.

**Imprint the Brand:** Every pixel, animation, word, and interaction must embody the Artisan-Outlaw-Fierce Guardian identity defined in Brand.md.

**Fuel the Growth Engine:** The site must offer clear, low-friction paths to subscribe to a Stream (the North Star entry point) and to install the full Relay application.

**Maintain Technical Integrity:** Zero third-party tracking scripts. Zero cookies without explicit, functional purpose. Entirely self-hosted, static infrastructure. The site must be a proof point of the privacy guarantees, not a violator of them.

## 2. Site Architecture

The site operates as a full-viewport single-scroll page with two primary modes: **The Hero State** and **The Scroll**. A deep-linked Stream landing page (`/stream/{id}`) exists as a lightweight entry point for the growth loop.

### 2.1 State 1: The Hero State (Default View)

The visitor is greeted by a full-viewport 3D WebGPU scene (with graceful fallback to WebGL 2.0 and a pre-rendered video loop).

The GLB gearbox model idles in warm-lit amber-dark space. Cream Oxide text fades in above it with the tagline and a brief pitch.

Minimal overlay UI: navigation header, tagline, and two primary buttons — "Install Relay" and "See It in Action".

Goal: Imprint the brand visually and provide immediate, frictionless paths to engagement (install or learn more).

### 2.2 State 2: The Scroll

Below the hero, content sections scroll seamlessly as the gearbox visual remains fixed or compresses into a persistent background element.

A "See It in Action" section replaces the interactive demo. It plays a pre-recorded, looping capture→enrich sequence that demonstrates a real Relay session with the actual Qwen 3.5 output, overlaid with the privacy proof monitor showing zero external requests.

Goal: Educate, prove privacy, convert, and invite visitors into the Stream subscription network.

### 2.3 Stream Landing Page (`/stream/{id}`)

A dedicated, lightweight page for shared Stream links.

Skips the interactive hero but retains the gearbox visual as a subtle background.

Displays the Stream's title, curator name with Provenance Seal, description, and a prominent "Subscribe" / "Get Relay" call-to-action.

Serves as the primary top-of-funnel entry from social shares and referrals.

## 3. The Hero + See It In Action

### 3.1 Hero Visual Scene

Scene: A pristine, warm-lit 3D planetary gearbox floating in deep `#1A1A1A` space. Tiny amber particles drift like suspended oil droplets.

Central Element: The Relay Gear logo-form rendered in 3D (GLB). Inner hub sealed and solid. Outer relay nodes pulsing softly with an amber glow.

Lighting: Single warm key light, subtle rim light in oil-slick iridescent (`#4B0082` to `#2E8B57`). Shadows are sharp but not harsh, rendered with PBR materials.

Materials: Brushed gunmetal (core), polished amber (contact points), dark anodized finish (inner hub).

Idle Animation: The gearbox rotates slowly on its Y-axis. A gentle Z-axis oscillation simulating inertia. The outer ring pulses with amber light at 4-second intervals.

Overlay Text: Cream Oxide (`#F5F0E8`) tagline fades in centered above the gearbox: *"Your private transmission for the age of on-device intelligence."*

Two buttons below the tagline:

- **Primary:** [ Install Relay ] → Scrolls to the `#download` section.
- **Secondary:** [ See It in Action ] → Scrolls to the `#action` section below the fold.

### 3.2 See It In Action Section

Trigger: Activated by scrolling to the `#action` anchor, either via the hero button or natural scrolling.

Layout: A full-width section with the gearbox visual reduced to a persistent background element on the left or bottom-right. The main content occupies the center.

Recording: A pre-recorded, high-fidelity capture of a real desktop Relay session:

1. **Setup:** A mock browser window or code editor frame fills the center of the section.
2. **Capture:** The user selects text in the frame. A cursor highlights the text. A subtle indicator shows: *"Captured with timestamp + source metadata."*
3. **Enrichment Phase:** The screen shows a brief animated transition (the text flowing as particles into a stylized gearbox icon). A non-intrusive toast reads: *"Processing on your device..."*
4. **Output:** Tags emerge as machined metal labels. Summary appears as an engraved plate. Outputs are the actual Qwen 3.5 enrichment from a real Relay session (not faked).
5. **Privacy Proof Overlay:** A small, fixed-position monitor in the corner of the recording shows a green indicator and the count *0 external requests* throughout the entire sequence.
6. **Looping:** The recording auto-restarts on completion. The user can replay, pause, or scrub the recording manually.
7. **Labeling:** The recording is clearly labeled: *"A real capture processed by Relay on-device AI."* This transparency eliminates any sense of deception.

Fallback: If autoplay is blocked by the browser, replace the recording with a static hero image of the enrichment output and a prominent "Click to play" overlay.

### 3.3 Privacy Proof Statement

Immediately below the recording, text fades in as an engraved metal plaque:

*"Everything you just saw happened inside your machine. We never saw your text. We couldn't if we tried."*

### 3.4 Post-Action Call to Action

Two options appear beneath the recording and privacy statement:

- **Primary:** [ Get the full Gearbox ] → Scrolls to the `#download` section.
- **Secondary:** [ Subscribe to a Stream ] → Opens the Community Streams panel inline, allowing visitors to enter their email and subscribe to a curator's public feed without installing anything. This directly activates the growth loop.

## 4. Scroll Behavior & The 3D Gearbox

The 3D gearbox scene remains live in the background as the user scrolls through content sections. It compresses to a smaller position (e.g., fixed bottom-right) on scroll, continuing its idle animation.

The initial full-viewport hero is not a gate — content sections are accessible immediately via scroll. Clicking "See It in Action" scrolls to the recording section. Clicking "Install Relay" scrolls to the download section.

Goal: Educate, convert, and invite visitors into the Stream subscription network without forcing interaction with the 3D scene.

## 5. Content Sections (The Reveal)

### 5.1 Why Now

**Headline:** "The age of the personal transmission has arrived."

**Body:** A short, urgent narrative that frames the broader shift. Avoids marketing hype; states facts.

> In 2026, on-device AI is no longer a promise—it's a platform shift. Apple ships a 3B-parameter model running locally on iPhones by default. Google releases capable open-source models small enough for phones. The EU AI Act demands data sovereignty. Zero-click searches have made the old web unviable. The tools of thinking are moving from the cloud to the edge. Your context belongs to you, not a server. Gearbox Relay is the transmission for this new reality.

**Visual:** A timeline graphic showing the convergence of hardware (Apple Foundation Models, Gemma 4), regulation (EU AI Act enforcement), and behavior (zero-click search dominance).

### 5.2 Philosophy

**Headline:** "Your mind, your transmission."

**Body:** A concise manifesto. No passive voice, no "we believe" statements.

> Gearbox Relay is the private operating system for your context. It captures what you notice, enriches it on your device, and lets you relay your curated perspective to others. No cloud ever touches your thoughts. No one can look inside. This is your personal transmission, built for the shift from Personal Knowledge Management to Personal Context Management.

**Visual:** A diptych—a hand-annotated blueprint of a gearbox next to a neural network diagram. Caption: *"Transmission is biological. Biology is mechanical."*

### 5.3 How It Works

**Headline:** "Capture. Enrich. Relay."

**Layout:** Three blueprint-style flow diagrams, each with a mechanical part label and a short description.

- **Capture: Intake Valve** — *"Highlight, copy, screenshot. Your attention leaves a trace. Relay grabs it with source metadata."*
- **Enrich: Gearbox Mesh** — *"On-device AI instantly tags, summarizes, and connects it to your existing knowledge. No cloud, zero token cost."*
- **Relay: Signal Out** — *"Publish curated Streams. Others subscribe to your signal. Your private library stays private. You decide what to transmit."*

### 5.4 The Five Guarantees (Trust Certification)

**Headline:** "The Transmission Guarantee."

**Layout:** Five statements styled as a stamped mechanical certification plate, using monospace and a seal-like border.

1. All AI enrichment runs on-device.
2. Sync data is end-to-end encrypted; the server sees only ciphertext.
3. Stream publications are opt-in; your private library is never exposed.
4. Zero third-party data sharing. No data to sell.
5. The core client is open-source (Apache 2.0). Independent verification is invited.

**Link:** A small link to the public sync protocol specification and the open-source repository.

### 5.5 Features (Interactive Spec Sheet)

**Layout:** A responsive grid of feature cards styled as machined part specifications.

**Card Design:** Monospace labels mimicking engineering specs: *Model: Themed Review, Torque: Spaced Repetition, Meshing: Semantic Search.* A toggle expands each card to reveal a plain-language description.

**Key Features Highlighted:**

- On-Device AI Enrichment
- Local-First Full-Text & Semantic Search
- Stream Publishing & Subscription
- Themed Review Sessions (AI-curated spaced repetition)
- Zero Token Cost, Zero Cloud Inference
- Open-Source Core (Apache 2.0)
- Data Export (Markdown, JSON, ZIP)
- Multi-Device Encrypted Sync

### 5.6 Pricing

**Headline:** "Simple fuel for your transmission."

**Layout:** Three tiers presented as physical gauge clusters.

- **Free:** Gauges show "3 Streams," "Unlimited Highlights," "Basic AI." Needle in a green zone. Large text: **$0**.
- **Pro:** Gauges show "Unlimited Streams," "Advanced AI," "Themed Reviews," "5GB Sync." Large text: **$8/month** or **$80/year**.
- **Founding Curator:** A special brass-badge treatment. Large text: **$60/year**, with a note: *"For the first 5,000 builders. Grandfathered for life."* A prominent "Join the Foundry" button.

**Creator Fee Note:** A small, honest footnote below the pricing gauges:

> *"Creators keep 100% of subscriber revenue for the first 12 months. After that, a voluntary Transmission Fuel contribution (10-20%) keeps the relay running. See our public cost ledger."*

(The cost ledger is a separate, lightweight page showing aggregate platform costs and average contribution rates.)

### 5.7 Community Streams (Live Growth Engine)

**Headline:** "Signals worth subscribing to."

**Layout:** A 3-column grid of featured Stream cards.

**Card Content:** Curator name with a Provenance Seal (a small gear icon with a duration stamp, e.g., "6 months in relay"), Stream title, short description, and a Subscribe button.

**Subscription Action:** Clicking Subscribe opens a minimal form (email only). After submitting, a confirmation message: *"You're tuned in. When new signals arrive, you'll hear about it. No spam, ever."* The user is then prompted to install the app for the full experience.

**Curator Call-to-Action:** A small link: *"Are you a curator? Publish your Stream on Relay."* Leads to the download/beta sign-up section.

### 5.8 Roadmap

**Headline:** "What's next in the transmission."

**Visual:** A zoomable blueprint with mechanical revision blocks.

**Items:**

- Public Beta — Status: In Progress
- Mobile Launch (iOS & Android) — Status: In Progress
- Collaborative Streams — Status: Planned
- Voice Capture — Status: Planned
- Browser Extensions (Chrome, Safari) — Status: Planned
- Federated Personalization (DP-FedLoRA) — Status: Research

No specific dates, but a clear sense of direction.

### 5.9 Footer / The Shop Floor

**Links:** GitHub Repository, Privacy Policy, Transmission Fuel Ledger, Contact (a minimal, privacy-respecting form or email address).

**Statement:** *"Hand-built, individually tested. No user data, no cloud, no compromises."*

**Copyright:** Gearbox.Ai

## 6. Conversion Flows & Growth Mechanics

### Flow 1: See It In Action → Product Sign-Up

1. User watches the recording or scrolls past it → CTA *Get the full Gearbox* → scrolls to a `#download` section.

### Flow 2: See It In Action → Stream Subscriber

1. User watches the recording → CTA *Subscribe to a Stream* → Community Streams panel opens inline.
2. User enters email to subscribe to one or more Streams. This is the primary low-friction growth loop entry.

### Flow 3: Direct Scroll (Bypassing the Recording)

For returning visitors or those who scroll immediately, the recording is not forced. The gearbox idles in the background, and content sections are accessible immediately. A "See It in Action" button remains available in the hero.

### Flow 4: Stream Deep Link (`/stream/{id}`)

1. User arrives from a shared Stream link.
2. Sees a dedicated page with the Stream's full details and a prominent *Subscribe* or *Get Relay* button.
3. This page loads quickly (no 3D scene requirement, just the gearbox as a static background image) and is optimized for social sharing metadata (Open Graph, Twitter Cards).

### Growth Instrumentation (Privacy-Preserving)

All conversion events (action section viewed, stream subscribed, app downloaded) are tracked using a self-hosted, privacy-first analytics instance (e.g., Plausible or a custom lightweight counter).

Absolutely no cookies, no fingerprinting, no external analytics scripts. Only anonymized, aggregate counts.

K-factor events for the product are tracked in-app, not on the website. The website's job is to feed the top of the funnel with trust and subscriptions.

## 7. Technical Architecture

### 7.1 Core Stack

- **3D Rendering:** WebGPU (primary) → WebGL 2.0 (fallback via TSL) → Pre-rendered video loop (final fallback).
- **3D Library:** Three.js with WebGPURenderer, using React Three Fiber for declarative scene management if using a React-based static site generator.
- **Recording Playback:** An HTML5 video element or a lightweight custom player for the pre-recorded capture→enrich sequence. The video is embedded as a WebM/H.264 file with a poster image fallback.
- **Privacy Proof Overlay:** A CSS-styled badge overlaid on the recording area showing "0 external requests" during playback. This is purely a visual overlay component aligned with the recording's timeline.
- **Hosting:** Static site deployed to a CDN (e.g., Cloudflare Pages, Netlify). Strict Content-Security-Policy headers.
- **Stream Pages:** Statically generated at build time or on-demand via a lightweight serverless function that reads from the public Stream metadata API.

### 7.2 Performance Budget

- **First Contentful Paint:** <1.5 seconds (static layout with loading indicator).
- **Time to Interactive:** <5 seconds for the gearbox scene to load and the hero text to fade in.
- **Recording Load Time:** <3 seconds for the pre-recorded video to be ready for playback (lazy-loaded below the fold).
- **Total Page Weight:** <500 KB (excluding the GLB file and recording video).

### 7.3 SEO & Crawler Strategy

The hero is a JavaScript-heavy experience. For search engine crawlers and social media bots, we serve a pre-rendered static HTML version of all content sections.

This is achieved via:

- Build-time pre-rendering for the main content sections.
- User-agent detection at the CDN level: if the user agent is a known crawler, serve the fully pre-rendered static page. If it's a real user, serve the immersive experience.

Stream landing pages (`/stream/{id}`) are fully static and SEO-optimized, containing all text content and metadata.

## 8. Implementation Phases

### Phase 1: Core Site & Essential Content (Launch)

- **Hero:** WebGPU gearbox idle animation with tagline and two primary CTAs ("Install Relay" / "See It in Action").
- **Fallbacks:** WebGL 2.0 render path, pre-rendered video loop for non-3D browsers.
- **See It In Action:** Pre-recorded capture→enrich sequence with real Qwen 3.5 output, privacy overlay, and clear labeling.
- **Content Sections:** Why Now, Philosophy, How It Works, The Five Guarantees, Pricing, Footer.
- **Conversion:** *Get the full Gearbox* link to a download/beta email capture section. Basic Stream subscription (email capture) with one or two featured curators.
- **Technical:** Static site deployment, CSP headers, self-hosted anonymized analytics.

### Phase 2: Full Scroll Experience & Growth Engine (Post-Launch Iteration)

- **Content Sections:** Features (interactive spec cards), Community Streams (live grid), Roadmap (zoomable blueprint).
- **Stream Deep Links:** Fully functional `/stream/{id}` pages with Open Graph metadata.
- **Visual Polish:** Refined scroll-triggered animations, workshop sound design finalized.

### Phase 3: Continuous Evolution (Ongoing)

- **Advanced Engagement:** Allow visitors to publish a one-off, anonymous, ephemeral Stream directly from the website as a taste of the full product.
- **Real-Time Data:** Display live subscriber counts and trending Streams on the Community Streams section.
- **Performance Optimization:** Based on real-user monitoring (anonymized), further optimize 3D scene and recording delivery.

## 9. Edge Cases, Errors & Accessibility

**Recording Autoplay Blocked:** If the browser blocks autoplay, display a static poster image of the enrichment output with a prominent "Click to Play" overlay. The recording starts on first interaction.

**Web Worker Used for Background Rendering:** The 3D scene runs in the main thread with offscreen rendering paths where available.

**Accessibility:**

- All text overlays and results are fully keyboard-navigable and readable by screen readers.
- The hero section is marked as `aria-hidden="true"`; all essential information is duplicated in accessible text elements.
- Color contrast between Cream Oxide (`#F5F0E8`) text and Gunmetal (`#1A1A1A`) background meets WCAG AAA standards.

## 10. Maintenance & Governance

**Recording Update:** The See It In Action recording can be re-captured at any time to reflect the latest product state. The recording file is swapped on the CDN independently of the site deployment.

**Stream Data:** Featured Streams on the Community section are curated manually by the Gearbox team or pulled from a simple API endpoint that lists public, opt-in Streams.

**Cost Ledger:** The public Transmission Fuel page is a separate, static page updated monthly with aggregate infrastructure costs and average Creator contribution rates. This is a manual update process for transparency.

**Brand Alignment Review:** This Website.md document should be reviewed quarterly alongside Brand.md and Monetization.md to ensure continued coherence as the product evolves.

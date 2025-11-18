# **MASTER SPEC — ResearchBase Desktop Application**

*A desktop-first, AI-orchestrated research environment.*

---

# **0. Product Purpose & Philosophy**

ResearchBase is a desktop application that helps researchers:

* organize, understand, and expand their literature,
* maintain structured knowledge with long-term AI memory,
* write and revise academic papers,
* learn and master the material through interactive sessions,
* operate in a minimal, chat-first environment.

The system is designed so that the **AI handles complexity**, and the UI remains simple, clean, and user-friendly.

---

# **1. Multi-Base Model**

A user may create **one or more Paper Bases** (“Research Bases”) at any time.

A Paper Base represents a **distinct research domain**, e.g.

* “Cognitive Science Research Base,”
* “Neuroscience Base,”
* “ML & AI Base.”

Most users will use **one big Base**, but multiple Bases are supported with minimal overhead.

Each Base has its own:

* Paper collection
* Categories
* Reports
* Writing projects
* AI-layer memory

The user selects the active Base upon opening the app.

---

# **2. Two Onboarding Pathways**

When creating a new Paper Base, the app asks:

> **How would you like to begin?**
>
> * **A. I already have papers to import.**
> * **B. I don’t have papers yet — help me build a Paper Base.**

### **Path A – User has papers**

1. User selects a PDF folder or library export (Zotero/EndNote optional).
2. App ingests PDFs:

   * metadata extraction
   * text extraction
   * embedding generation
3. AI proposes categories.
4. User can rename / merge / split.
5. AI generates:

   * Category Reports
   * Global HTML Report
   * Key Figure Gallery for important papers (optional per user approval)

### **Path B – User has no papers**

1. Conversational interview about:

   * research area,
   * questions,
   * interests,
   * level of expertise.
2. AI proposes a starter set of open-access papers.
3. User approves or rejects.
4. AI builds categories, summaries, and reports.
5. User is informed of next steps (“learn,” “write,” “expand library,” etc.).

---

# **3. Minimal Desktop UI**

The UI is intentionally simple:

### **Visible Components**

* **Chat Panel** — the primary interface.
* **Paper Base Browser** (left sidebar):

  * Bases
  * Categories
  * Papers (with “Open PDF”)
* **Writing Projects** (list + open project)
* **Settings** (API key, LaTeX path, theme, audio input)

### **No complex views**

Complexity lives in chat + reports, not in UI panels.

### **Reports are HTML files**

* Not rendered as a heavy UI component.
* Open in browser.
* Easily shareable.

---

# **4. Dual-Layer File Architecture**

All data is separated into:

# **4.1 User Layer (Visible)**

Readable artifacts meant for human consumption.

Location example: `/User/<BaseName>/`

Includes:

* Original PDFs
* LaTeX projects (`.tex`, `.bib`, `.pdf`)
* HTML reports
* Exports (markdown, PDF, images)
* Figures extracted from papers (if approved by user)

Users control and see these files.

---

# **4.2 AI Layer (Invisible)**

Machine-facing internal documentation used for memory, orchestration, and reliable context loading.

Location example: `/AI/<BaseName>/`

Contains:

* Paper metadata index (`papers_index.json`)
* Category structure (`category_map.json`)
* Embeddings (vector store)
* Profiles:

  * `UserProfile.md`
  * `WritingProfile.md`
  * `WorkProfile.json`
  * `KnowledgeProfile.json`
* Conversation logs (summarized sessions)
* AI tasks, suggestions, todos
* Temporary context blocks

The user does not interact with these files.

The AI never depends on PDFs/HTML for memory — only AI-layer docs.

---

# **5. Core Features**

## **5.1 Paper Base Ingestion**

* Parse PDFs for text and metadata.
* Identify DOIs.
* Extract figures (optional per user approval).
* Store embeddings for semantic search.
* Generate AI summaries:

  * abstract rewrite,
  * 1-sentence insight,
  * method summary,
  * findings summary,
  * limitations summary.

---

## **5.2 Automatic Categorization**

The AI clusters papers and proposes category names.

User can:

* rename categories,
* merge/split,
* move papers.

The system updates AI-layer documentation accordingly.

---

## **5.3 Reports (HTML)**

Each Base has:

### **(A) Global Report**

Contains:

* overview of categories,
* summaries,
* key themes,
* paper list with 1-sentence insights,
* “Important Figures Gallery” (optional per user),
* suggested future readings,
* conceptual map (optional),
* timeline view (optional),
* hotspots / gaps in the field.

### **(B) Category Reports**

Contain:

* category description,
* key papers,
* insights,
* representative figures (user-approved),
* relationships to other categories.

### **(C) Mini-Reports (on demand)**

User can request targeted reports on:

* specific topics,
* subsets of papers,
* methods,
* research questions.

---

# **6. Chat Assistant (Core Interaction)**

The chat is the primary interface for:

* asking questions about literature,
* triggering tasks,
* generating reports,
* starting writing sessions,
* starting learning sessions,
* requesting new papers,
* refining categories,
* editing writing style,
* exploring concepts.

The AI:

* is proactive,
* informs the user of possibilities,
* suggests next steps,
* clarifies features,
* asks permission before heavy operations,
* keeps conversations organized in the AI-layer.

---

# **7. AI Profiles (Long-Term Memory)**

### **7.1 UserProfile**

* field + subfields
* background
* expertise level
* preferences for depth/complexity
* writing tone preferences
* common mistakes

### **7.2 WorkProfile**

* current research goals
* active writing projects
* reading intentions
* open TODOs

### **7.3 WritingProfile**

* stylistic parameters
* favorite papers as style models
* author’s preferred structure
* examples of good/bad writing (stored as references)

### **7.4 KnowledgeProfile**

Tracks concept mastery:

* concepts extracted from literature
* coverage of those concepts
* difficulty level for the user
* weaknesses and misunderstandings

Used in learning sessions and planning.

---

# **8. Writing Assistant (LaTeX)**

### **Capabilities**

* Style interview to determine tone, structure, and preferences.
* Outline creation based on writing goal.
* Draft generation in LaTeX.
* `.tex` and `.bib` management.
* PDF compilation.
* Inline edits via chat.
* Integration of citations from the Paper Base.
* Ability to analyze the user’s favorite papers as “style models.”

### **User-visible artifacts**

Stored under `/User/<BaseName>/WritingProjects/<Project>/`.

---

# **9. Learning Mode**

Modes:

* **Quiz**
* **Oral Exam**

The AI:

* selects questions based on user’s KnowledgeProfile,
* evaluates answers,
* corrects misunderstandings,
* updates KnowledgeProfile,
* tracks progress over time.

User can choose to focus on:

* entire Paper Base,
* categories,
* specific papers,
* specific concepts.

---

# **10. Figure Handling**

### **Figure Extraction (optional)**

* The AI offers to extract important figures from papers.
* User sees a proposed list and can approve individually.

### **Figure Interpretation**

The AI uses extracted figures + captions to:

* summarize trends,
* explain results,
* identify key visual information.

### **Figure Embedding in Reports**

* Category Reports may include representative figures.
* Global Reports may include a Key Figure Gallery.

---

# **11. Visualization Features (Optional but Supported)**

The AI may generate:

* conceptual maps,
* topic clusters,
* citation networks,
* similarity networks,
* historical timelines,
* hotspot maps (research gaps),
* relevance heatmaps for writing projects.

These appear inside HTML reports only.

User chooses whether to include them when generating reports.

---

# **12. Paper Discovery (On-Demand)**

The AI informs the user:

> “I can help you find new papers.”

Options:

* search based on user’s current research,
* search based on missing categories,
* search for latest publications,
* search for methodological diversification.

AI asks:

> “Do you have something specific in mind, or should I search broadly?”

No periodic auto-harvesting unless added in the future.

---

# **13. Voice Chat (Optional)**

User can enable audio input.

Used especially in:

* oral exam mode,
* brainstorming,
* writing feedback.

---

# **14. Intent Routing & Orchestration**

The system uses natural-language intent detection to route user commands to:

* Paper Base actions,
* Report generation,
* Profile updates,
* Writing actions,
* Learning sessions,
* Mini-reports,
* Paper discovery,
* Category edits.

The LLM always:

* proposes available actions,
* confirms before executing heavy operations,
* keeps AI-layer documentation in sync.

---

# **15. Non-Functional Principles**

* **Privacy-first**
  All user data stays local. Only prompt excerpts required for LLM calls are sent out.

* **Minimal interface**
  A simple left panel + chat. No unnecessary UI features.

* **Regenerability**
  HTML reports can be regenerated anytime from AI-layer docs.

* **Predictability**
  All AI memory kept in structured JSON/MD.

* **Extensibility**
  New features fit naturally via the AI-layer + chat interface.

---

# **16. Roadmap for Spec Kit Development**

1. **Spec 00 — Master Spec (this document)**
2. **Spec 01 — Onboarding (Two-Path Flow) + Multi-Base Creation**
3. **Spec 02 — Paper Ingestion + Metadata + Figures**
4. **Spec 03 — Categorization + Editing**
5. **Spec 04 — Reports (HTML) + Visualizations**
6. **Spec 05 — AI Profiles (User/Work/Writing/Knowledge)**
7. **Spec 06 — Chat Assistant + Intent Routing**
8. **Spec 07 — Writing Assistant (LaTeX)**
9. **Spec 08 — Learning Mode**
10. **Spec 09 — Paper Discovery**
11. **Spec 10 — Voice Input**
12. **Spec 11 — UI and Settings Finalization**

---


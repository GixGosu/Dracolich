# Multi-Agent AI Research Briefs

Create a collection of 50 interactive research briefs on the most important topics in multi-agent AI systems.

## Phase 1: Topic Discovery

Research and identify **100 candidate topics** related to multi-agent AI. Sources to explore:
- Recent academic papers (arXiv, NeurIPS, ICML, ICLR 2024-2026)
- Industry research blogs (Anthropic, OpenAI, DeepMind, Google Research, Meta AI)
- Open source projects (AutoGPT, CrewAI, LangGraph, AutoGen, etc.)
- AI safety and alignment research
- Production deployments and case studies

Topic categories to cover:
- **Architecture**: Agent communication, memory, planning, tool use
- **Coordination**: Task decomposition, delegation, consensus, conflict resolution
- **Governance**: Safety, oversight, alignment, human-in-the-loop
- **Execution**: Parallelism, resource management, error handling, recovery
- **Evaluation**: Benchmarks, metrics, testing multi-agent systems
- **Applications**: Research, coding, analysis, creative work, automation

Each candidate topic needs:
- Topic name (concise, specific)
- One-sentence description
- Why it matters (relevance score 1-10)
- Current state (emerging/active/mature)
- Key papers or projects (2-3 references)

## Phase 2: Topic Selection

The research team must **vote to select the top 50 topics** from the 100 candidates.

Voting criteria:
1. **Impact** (30%): How significant is this for the field?
2. **Timeliness** (25%): Is this actively being worked on now?
3. **Depth** (20%): Is there enough substance for a meaningful brief?
4. **Breadth** (15%): Does this connect to other important topics?
5. **Actionability** (10%): Can practitioners apply insights from this?

Selection process:
- Each voting agent scores topics independently
- Scores are aggregated with weighted criteria
- Top 50 by aggregate score are selected
- Ties broken by Impact score

Document the voting results with:
- Final ranked list of 50 topics
- Score breakdown for top 50
- Notable exclusions and why they didn't make the cut

## Phase 3: Brief Creation

For each of the 50 selected topics, create an **interactive HTML brief**.

Each brief must include:

### Content Sections
1. **Overview** (100-150 words): What is this topic and why does it matter?
2. **Key Concepts** (3-5 bullet points): Core ideas a reader must understand
3. **Current Approaches**: How are researchers/practitioners tackling this?
4. **Open Problems**: What remains unsolved or controversial?
5. **Notable Work**: 3-5 papers, projects, or implementations with links
6. **Connections**: How this relates to other topics in the collection
7. **Further Reading**: Curated resources for deeper exploration

### Interactive Elements
- Expandable/collapsible sections
- Topic tags for filtering
- Links between related briefs
- Search functionality (across all briefs)
- Progress tracker (briefs read)
- Dark/light mode toggle

### Visual Design
- Clean, readable typography
- Cyberpunk-inspired color scheme (optional nod to our roguelike)
- Mobile-responsive layout
- Syntax highlighting for any code examples
- Diagrams where helpful (can be ASCII/text-based)

## Deliverables

```
output/[project]/
├── index.html              # Main hub with all 50 briefs, search, filtering
├── briefs/
│   ├── 01-[topic-slug].html
│   ├── 02-[topic-slug].html
│   └── ... (50 total)
├── data/
│   ├── topics.json         # All 100 candidate topics with scores
│   ├── selected.json       # Top 50 with voting breakdown
│   └── connections.json    # Topic relationship graph
├── assets/
│   ├── styles.css
│   └── app.js              # Interactivity (search, filters, etc.)
└── README.md               # Project overview and methodology
```

## Success Criteria

1. **Coverage**: 50 briefs spanning all major topic categories
2. **Quality**: Each brief is substantive, accurate, and well-sourced
3. **Interactivity**: Search, filtering, and navigation work smoothly
4. **Connections**: Topics are meaningfully linked to related topics
5. **Usability**: A practitioner could use this as a reference resource
6. **Documentation**: Methodology is transparent (voting results, sources)

## Constraints

- All content must be factual and properly attributed
- No hallucinated papers or projects — only cite real work
- Briefs should be accessible to someone with basic ML knowledge
- Total bundle should work offline (no external API calls)
- Keep individual brief files under 50KB each

## Notes for Swarm Design

This task benefits from:
- **Parallel research**: Multiple agents exploring different source categories
- **Adversarial selection**: Voting forces quality filtering
- **Specialist roles**: Researchers, synthesizers, writers, designers
- **Integration phase**: Someone must wire the 50 briefs together coherently
- **QA verification**: Check links, validate citations, test interactivity

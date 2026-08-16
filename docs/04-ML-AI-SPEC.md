# NIKHIL//OS --- AI/ML Technical Specification

## 1. Goal

AI should be an operating-system capability rather than a decorative
chatbot.

The AI system must:

-   understand user intent
-   retrieve evidence
-   reason over structured knowledge
-   plan actions
-   call controlled tools
-   respect permissions
-   explain decisions

## 2. AI Runtime

``` text
User
 ↓
Intent
 ↓
Planner
 ↓
Retriever
 ↓
Tool Selector
 ↓
Permission Validator
 ↓
OS Service
 ↓
Result
 ↓
Explanation
```

## 3. Intent Engine

Initial intents:

-   SEARCH_PROJECTS
-   SEARCH_SKILLS
-   SEARCH_EXPERIENCE
-   SEARCH_EVIDENCE
-   OPEN_APPLICATION
-   QUERY_GRAPH
-   SYSTEM_STATUS
-   JOB_ANALYSIS
-   CAREER_ANALYSIS

Start with deterministic routing and structured classification.

Introduce a learned classifier only after establishing a baseline.

## 4. Knowledge Graph

Entities:

``` text
Person
Skill
Technology
Project
Experience
Contribution
Achievement
Certification
Organization
Claim
Evidence
Event
```

Relationships:

``` text
HAS_SKILL
USES
BUILT
WORKED_AT
CONTRIBUTED_TO
DEMONSTRATES
SUPPORTED_BY
RELATED_TO
```

## 5. RAG

Pipeline:

``` text
Query
 ↓
Query normalization
 ↓
Intent/entity extraction
 ↓
Hybrid retrieval
 ↓
Vector search
 ↓
Graph retrieval
 ↓
Reranking
 ↓
Evidence filtering
 ↓
LLM generation
```

## 6. Evidence

Every important claim must have:

-   source
-   entity
-   confidence
-   retrieval path

UI action:

**Why this result?**

Response:

``` text
Semantic similarity: 0.91
Visitor relevance: 0.87
Evidence strength: 0.82

Evidence:
- WhatBytes
- CompanyMind
- Open-source contributions
```

## 7. Visitor Interest Model

Session-only vector:

``` text
backend
AI
ML
systems
mobile
open-source
frontend
databases
```

Update from:

-   navigation
-   searches
-   graph interactions
-   selected roles
-   project views

Avoid identity inference.

## 8. Recommendation

Baseline:

``` text
score =
semantic_similarity
+ visitor_interest
+ evidence_strength
+ relevance
+ recency
```

Later evaluate a learned ranker.

Evaluation:

-   Recall@K
-   Precision@K
-   MRR
-   CTR
-   time-to-relevant-content

## 9. Job Analyzer

Pipeline:

``` text
Job Description
 ↓
Text extraction
 ↓
Skill extraction
 ↓
Skill normalization
 ↓
Semantic matching
 ↓
Evidence retrieval
 ↓
Gap analysis
 ↓
Match score
```

Output:

-   strong matches
-   partial matches
-   gaps
-   supporting evidence

## 10. Career Simulator

Inputs:

-   target role
-   desired specialization
-   desired technologies

Output:

-   current strengths
-   gaps
-   relevant existing evidence
-   possible projects
-   learning recommendations

It must clearly state that this is a simulation.

## 11. AI Tool Security

Never expose:

-   arbitrary shell
-   unrestricted filesystem
-   database credentials
-   arbitrary JavaScript
-   deployment controls

Tools should be allowlisted and schema validated.

## 12. Agent Observability

Record:

``` text
request
intent
retrieval
tools
permissions
actions
result
latency
errors
```

Expose this through Developer Mode.

## 13. Evaluation

Create a test dataset covering:

-   factual questions
-   ambiguous questions
-   project search
-   evidence search
-   job matching
-   adversarial prompts

Measure:

-   retrieval quality
-   unsupported claims
-   tool accuracy
-   latency
-   failure rate

## 14. Model Independence

Keep model access behind a provider interface.

The rest of the system should not depend directly on one model vendor.

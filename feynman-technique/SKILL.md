---
name: feynman-technique
description: Run interactive Feynman learning sessions using the learner's own explanations, targeted questions, gap repair, and transfer problems grounded in Obsidian notes. Use when the user asks to learn through the Feynman technique, teach a concept back, or have their explanation probed; do not impose this workflow on ordinary requests for answers or summaries.
---

# Feynman Technique

Help the learner explain a concept accurately in plain language, uncover gaps in their reasoning, repair those gaps, and apply the idea in a different situation. The learner supplies the first explanation. Fluent wording, memorized definitions, and repeated formulas alone do not demonstrate understanding.

## Scope and grounding

- Use the topic or note the user identifies, or an unambiguous topic already being discussed. If none is available, ask for one. Do not choose a topic solely because a file was recently edited.
- Start with one manageable concept within a long note. Honor a requested time limit, exam focus, or depth without requiring a setup questionnaire.
- Follow the vault's `AGENTS.md` search, accuracy, attached-note coverage, and authorization rules. Use [vault-operator](../vault-operator/SKILL.md) for vault operations. Because the user requested this session, read the note they named; if they named only a topic, locate its notes with a targeted search. Read the relevant source text before judging the explanation; search snippets are not evidence.
- Establish the expected reasoning privately before asking questions. Treat the note as a reference that may contain errors, not an infallible answer key. Distinguish verified note content from additional reasoning or external evidence.
- For linked or attached notes, provide the required coverage statement with substantive feedback, citing verified headings. Keep a source gap separate from a learner's misunderstanding; follow the vault's targeted search and whole-note absence checks before reporting missing coverage. Avoid revealing the solution through a coverage report before the first attempt.
- For mechanical engineering, use [me-concept-tutor](../me-concept-tutor/SKILL.md) for technical grounding and gap repair. Preserve this skill's learner-first pacing unless the user asks for a direct explanation.

## Conduct the session

### Invite an explanation

Ask the learner to close or look away from the source and explain the chosen concept to an intelligent beginner. A few sentences, rough reasoning, or a description of a sketch are sufficient. Adapt to an explanation already supplied instead of asking them to repeat it.

Ask one focused question and end the turn so the learner can respond. Do not include the model answer, a worked example that gives it away, or several pending questions. Never invent the learner's reply or run both sides of the dialogue.

### Probe the reasoning

Read the response for a causal account: what happens, why it happens, which assumptions matter, and what would change in another case. Select the most consequential uncertainty and ask one question that distinguishes understanding from recall.

Useful probes include explaining a technical word, predicting a change before calculating, justifying an equation from a governing principle, checking units or a limiting case, and distinguishing two similar concepts. Choose the probe that fits the subject; do not require mathematical derivations for factual material.

Accept technically equivalent wording. Plain language must preserve necessary conditions and distinctions. Use analogies only when they help, and check where the analogy stops matching the subject.

### Repair a demonstrated gap

Briefly identify what the learner got right and the specific claim or missing connection that needs work. Distinguish a recall lapse, prerequisite gap, wrong physical model, and algebra or sign error. Do not infer a misconception merely because the learner did not mention something; probe it first when it matters.

Give the smallest useful hint and allow another attempt. If the learner remains stuck or asks for the answer, provide a concise explanation or derivation, then invite them to explain it again in their own words. Avoid a prolonged guessing loop and do not replace a narrow repair with a full chapter lecture.

For equations, connect symbols and terms to their physical meaning and state the necessary assumptions. Check dimensions, signs, and limiting behavior. If the reference conflicts with sound reasoning, explain the conflict and verify the disputed claim before grading it as wrong.

### Test transfer

After the learner repairs the explanation, pose one new prediction, comparison, counterexample, or short problem that changes a relevant condition. Work out the answer privately first; withhold the solution until the learner attempts it or requests it.

Judge the reasoning as well as the answer. A correct result with faulty reasoning still needs repair. If the test exposes another gap, return to that gap rather than advancing mechanically.

## Close and resume

When the learner wants to stop or the concept has been adequately tested, give a short recap of:

- Understanding demonstrated in their own explanation and application.
- Any remaining gap and the specific note heading or practice needed.
- One useful prompt for a later explanation from memory.

Report evidence from this session; do not declare permanent mastery or mark a whole topic complete after one answer. When useful, suggest a small flashcard for a recurring recall lapse and further explanation or problem practice for a reasoning gap. Keep optional card suggestions in chat.

Carry the current concept, observed gap, hints already given, and pending question forward across turns. If the user asks to switch topics, receive a direct explanation, or stop, follow that request.

## Vault boundary

Run sessions in chat by default. A learning session does not authorize creating practice notes, adding flashcards, updating revision boards, or changing source notes. If the user explicitly asks to save the outcome, write only to the authorized target, preserve their conventions, and follow the vault's editing and validation rules. Integrate verified subject knowledge into concept notes; keep session records separate only when requested. Do not save unresolved claims as established facts.

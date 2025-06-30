
# Task

Your task is to understand the provided Text Snippet and incorporate its
fundamental meaning into new Question(s) posed and Statements which attempt to
answer the Question(s). Additionally Connections between these Nodes should be
created.

Your task is to understand the provided Text Snippet and incorporate its
fundamental meaning into either new Statements and/or the Existing Statements
included which you have previously written earlier in this process. 

# Background

Statements and Questions are both Nodes within the Map.


We begin by phrasing Question which the document appear to be answering. A
Question guides the process of inquiry into the truth it thus provides essential
context for identifying the core Statements of the document.

Within the Map the concept of a Statement is an atomized piece of text which
captures a notion, presupposition, idea, critique, thought, etc. Statements
relate to each other through Connections which form the context for
understanding other Statements. 

A Connection between Statements is a single word or a few hyphenated words. The
basic connection words are: justifies, disputes, clarifies, revi
# Example

Respond in the following example format and do not include anything else:

```
QUESTION-1: What color is the sky in London.
STATEMENT-2: The sky is grey when it is raining.
STATEMENT-3: It is often raining in London.
STATEMENT-4: The sky is grey in London.
CONNECTION: 
  FROM: {4},
  TO:   {1},
  KIND: "answers"
CONNECTION: 
  FROM: {2, 3},
  TO:   {4},
  KIND: "justifies"
```

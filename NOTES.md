


# FILE TYPE:
 - should probably process an entire map within the local folder 
 - the map folder should also have the duck-db folder as well as the mou_config.toml file
 - referenced digested documents
 - also allow for mou_config.local.toml
 - a digested map could be stored as a series of NODES with connections as toml
   in a my_map.mou file 
    - there could also be a synthesis-mou file which is the connections and some
      statements between two map.mou files - the corresponding GLUE so to speak
      between maps.

# CONNECTION TYPES
 - Each connection has an associated direction 
    - NON      -----
    - one-way  ---->
    - AND      <--->
    - OR       <---- or ----> or <----->
       - any possibility of direction
    - XOR      <---- or ---->
       - there is an unknown mono-direction of either one way or the other, but
         not both
 - Connection types should only be a singular concept (can be hyphenated words)
    - basic connection types: 
       justifies, disputes, clarifies, revises, explains, example-of, summarizes,
       hypothetical-example-of, temporally-prior-to, temporally-simultaneous,
       distinct-from, synonymous-to (same), resolves, detracts-from,
       adds-doubt-to

# NODES
 - Can be a question or a statement
 - Each needs an ID which would be the hash of the statement
    - this hash would need to be updated with each node change during document
      digesting
 - Each statement should also have REFERENCE metadata associated with it. If it
   was digested from a document this metadata should be reference this AS well
   as 
 - Has associated tags - maybe these should just be in the form of hashtags at
   the end of the statement?! 
     - could be #time:1800 as a hash tag defining a contextual time period!
     - OR could move away from hashtags (political) to squiggle-tags ~somethingcool 
     - ACTUALLY probably just use tags: some-tag, some-other-tag

-----------------------
# Document Digesting

1. First pass:
2. Query small chunks for 
    - core question(s) answered in chunk
    - summary of the chunk
3. Order questions hierarchically
    - generate global guiding question(s)
    - use embedding network and nearby questions to determine possible connections between questions
    - analyze if these connections make sense using global summary and chunks from which those questions were generated
4. Generate Global summary from chunk summaries
    - feed in the Global Guiding Question(s) during this query
5. Second pass: Generating statements for each chunk, feed in: 
    - global summary
    - the previously generated question for this chunk
    - all questions up the hierarchical question chain leading to the 
      Global-Guiding Question(s)
6. Potentially perform step-4 multiple times (Configurable)
7. Query LLM to choose the best-statement set from 5 (if applicable
8. generate logical connections long-list
    - select a set of statements/questions which may have connections
       - select some by querying the embed for the statements
       - also select some by proximity of the source text snippet they were
         generated from.
    - prompt to try to capture all connections, even loose or
      potentially spurious ones, these will be pruned later
    - perform this prompt multiple times in with different statements and
      compile a long list of all connections generated.
9. (non-LLM) programmatically delete identical duplicate connections
10. Correct/verify of logical connections 
    - do NOT use chunk summaries in this prompt, all connections should be
      self-evident based on the statements themselves.
    - prompt with guiding questions for all relevant statements
    - querying reflection each connection individually (“is this correct if not
      provide correction; is this necessary to maintain understanding among
      related statements if not, delete it”
11. (non-LLM) Prune nodes disconnected from Global-Guiding Question 
12. Generate new outer questions/issues (New Questions)
     - Go through statements with connections and attempt generate new questions
       arising out of negative connections between statements such as disputes etc. 
     - Place these open New Questions on the Map but maintain a separate list of
       them for the next step
13. IF This step (Step-12) has not been taken more than "max_calls_step_12" then
    Query the document (traditional document RAG?) to see if the New Questions
    are addressed already within the document and just not adequately put into
    statements.
     - For each Question that IS addressed add the New Statement Node(s) to the
       map which address this question, Automatically add an "Addresses"
       connection type.
     - For each New Statement perform Steps 7-9 to create new Connections
       SPECIFIC to this New Statement (ignore all connections generated which do
       not include the New Statement)
     - For each new connection generated Perform step 11 to recursively generate
       New potential Questions
     - Increase num-times-step-12-called by one (to prevent infinite recursion) 
14. Consolidation:
     - Iterate through the Map and groups of connected Questions, Statements,
       and their Connections. 
     - Attempt to generate consolidation statement nodes which effectively
       succinctly summarize entire trains-of-thought/sections within the Map. 
        - maybe query the LLM in multiple passes and choose the best (similar to
          Steps 5-6)
     - Automatically add "consolidates" Connections to the map for all the
       sub-Statements consolidated by this new Statement
     - Each Consolidation Statement should inherit all the surrounding
       Connections to the sections of the map it Consolidates, We must refine
       and the Consolidation Statement to ensure that all these connections seem
       valid.
     - Consolidation Nodes will help collapse the Map visually but can be
       re-expanded as necessary if disagreements further contention is found

#### OLD THINKING
  1st iteration - iterate through each chunk in the document and query an LLM to either
   generate or modify existing statements AND question nodes
    - during this first pass the only connections which are to be created are
      between the generated question nodes and associated statement nodes
    - pass in relevant statements (say 10?) by doing a embed query of the existing statements 
    - ALSO just pass in a certain amount (say 10 more?) of recent or "salient" statements
    - LONG TERM: - for giant texts could also query FROM the recent statements
 - Set a "goal" number of statements which are to be generated for the text
   being digested
 - Once the number of statements reaches say 110% of the goal then attempt to
   condense/combine or cut 10% of the statements from the statement list
    - this could be done in an iterative fashion, looking at sections of
      statements coming up with a series of candidates for which statements
      should be cut or combined, then selecting between the candidates
 - Once a full pass of the text is completed generate a global summary FROM all
   of the statements passed in. 
 2nd iteration - NOW do a second pass digesting the document in a very similar way as the
   first pass but also include the summary of the document with each chunk, as a
   guided overall context for understanding each chunk. 
 Post-process - go through all the relationships by groups and attempt to define the
   connections between each of the nodes as well as generate new statement and
   question nodes which arise out of contraditions. further consolidation steps
   may also be taken here. 

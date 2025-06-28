


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

## 1st iteration
 - Chunk the document (DO NOT attempt to digest the document in one query lol!)
 - iterate through each chunk in the document and query an LLM to either
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

## 2nd iteration
 - NOW do a second pass digesting the document in a very similar way as the
   first pass but also include the summary of the document with each chunk, as a
   guided overall context for understanding each chunk. 

## Post-process
 - go through all the relationships by groups and attempt to define the
   connections between each of the nodes as well as generate new statement and
   question nodes which arise out of contraditions. further consolidation steps
   may also be taken here. 

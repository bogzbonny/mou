use {
    crate::{config::Config, templates::Templates, util::strip_markdown_tags},
    anyhow::Result,
    async_trait::async_trait,
    indoc::formatdoc,
    swiftide::{
        query::{
            self, answers, query_transformers, search_strategies::SimilaritySingleEmbedding,
            states, Query,
        },
        traits::{EmbeddingModel, EvaluateQuery, Persist, Retrieve, SimplePrompt},
    },
    swiftide_core::{document::Document, Answer},
};

#[tracing::instrument(skip_all, err)]
pub async fn query<S>(config: &Config, storage: &S, query: impl AsRef<str>) -> Result<String>
where
    S: Retrieve<SimilaritySingleEmbedding> + Persist + Clone + 'static,
{
    tracing::debug!(query = query.as_ref(), "querying config");
    // Ensure the table exists to avoid dumb errors
    let _ = storage.setup().await;

    let answer = build_nearest_pipeline(config, storage, None)?
        .query(query.as_ref())
        .await?
        .answer()
        .to_string();
    Ok(strip_markdown_tags(&answer))
}

/// Pipeline for simply querying the database for closest matches
pub fn build_nearest_pipeline<'b, S>(
    config: &Config,
    storage: &S,
    evaluator: Option<Box<dyn EvaluateQuery>>,
) -> Result<query::Pipeline<'b, SimilaritySingleEmbedding, states::Answered>>
where
    S: Retrieve<SimilaritySingleEmbedding> + Clone + 'static,
{
    let backoff = config.backoff;
    //let query_provider: Box<dyn SimplePrompt> =
    //    config.query_provider().get_simple_prompt_model(backoff)?;
    let embedding_provider: Box<dyn EmbeddingModel> =
        config.embedding_provider().get_embedding_model(backoff)?;

    let search_strategy: SimilaritySingleEmbedding<()> = SimilaritySingleEmbedding::default()
        .with_top_k(30)
        .to_owned();

    let answerer = ListAnswerer::default();

    let mut pipeline = query::Pipeline::from_search_strategy(search_strategy);

    if let Some(evaluator) = evaluator {
        pipeline = pipeline.evaluate_with(evaluator);
    }

    Ok(pipeline
        .then_transform_query(move |mut query: Query<states::Pending>| {
            let current = query.current();
            query.transformed_query(formatdoc! {"{current}"});
            Ok(query)
        })
        .then_transform_query(query_transformers::Embed::from_client(
            embedding_provider.clone(),
        ))
        .then_retrieve(storage.clone())
        .then_answer(answerer))
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ListAnswerer();

#[async_trait]
impl Answer for ListAnswerer {
    #[tracing::instrument(skip_all)]
    async fn answer(&self, query: Query<states::Retrieved>) -> Result<Query<states::Answered>> {
        let mut context = tera::Context::new();

        context.insert("question", query.original());

        let documents = query
            .documents()
            .iter()
            .map(Document::content)
            .collect::<Vec<_>>()
            .join("\n---\n");
        let answer = format!("Nearest statements are:\n\n{documents}");
        Ok(query.answered(answer))
    }
}

/// Builds a query pipeline
///
/// # Panics
///
/// Should be infallible
pub fn build_query_pipeline<'b, S>(
    config: &Config,
    storage: &S,
    evaluator: Option<Box<dyn EvaluateQuery>>,
) -> Result<query::Pipeline<'b, SimilaritySingleEmbedding, states::Answered>>
where
    S: Retrieve<SimilaritySingleEmbedding> + Clone + 'static,
{
    let backoff = config.backoff;
    let query_provider: Box<dyn SimplePrompt> =
        config.query_provider().get_simple_prompt_model(backoff)?;
    let embedding_provider: Box<dyn EmbeddingModel> =
        config.embedding_provider().get_embedding_model(backoff)?;

    let search_strategy: SimilaritySingleEmbedding<()> = SimilaritySingleEmbedding::default()
        .with_top_k(30)
        .to_owned();

    let prompt_template = Templates::from_file("agentic_answer_prompt.md")?;
    let document_template = Templates::from_file("indexing_document.md")?;

    // NOTE: Changed a lot to tailor it for agentic flows, might be worth upstreaming
    // Simple takes the retrieved documents, formats them with a template, then throws it into a
    // prompt with to answer the original question properly. It's really simple.
    let simple = answers::Simple::builder()
        .client(query_provider.clone())
        .prompt_template(prompt_template.into())
        .document_template(document_template)
        .build()
        .expect("infallible");

    let mut pipeline = query::Pipeline::from_search_strategy(search_strategy);

    if let Some(evaluator) = evaluator {
        pipeline = pipeline.evaluate_with(evaluator);
    }

    Ok(pipeline
        .then_transform_query(move |mut query: Query<states::Pending>| {
            let current = query.current();
            query.transformed_query(formatdoc! {"{current}"});
            Ok(query)
        })
        .then_transform_query(query_transformers::GenerateSubquestions::from_client(
            query_provider.clone(),
        ))
        .then_transform_query(query_transformers::Embed::from_client(
            embedding_provider.clone(),
        ))
        .then_retrieve(storage.clone())
        // .then_transform_response(response_transformers::Summary::from_client(
        //     query_provider.clone(),
        // ))
        .then_answer(simple))
}

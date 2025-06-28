use {
    super::garbage_collection::GarbageCollector,
    crate::{Config, Statements},
    anyhow::Result,
    async_trait::async_trait,
    dyn_clone::DynClone,
    swiftide::indexing::{transformers, Node},
    swiftide::traits::{EmbeddingModel, NodeCache, Persist, SimplePrompt},
};

#[async_trait]
pub trait Index: Send + Sync + std::fmt::Debug + DynClone {
    async fn query_statements(&self, config: &Config, query: &str) -> Result<String>;
    async fn index_statements(&self, statements: Statements, config: &Config) -> Result<()>;
}

// NOTE: Indexing in parallel guarantees a bad time

#[tracing::instrument(skip_all)]
pub async fn index_statements<S>(statements: Statements, config: &Config, storage: &S) -> Result<()>
where
    S: Persist + NodeCache + Clone + 'static,
{
    garbage_collect(config).await?;

    let backoff = config.backoff;

    let indexing_provider: Box<dyn SimplePrompt> = config
        .indexing_provider()
        .get_simple_prompt_model(backoff)?;
    let embedding_provider: Box<dyn EmbeddingModel> =
        config.embedding_provider().get_embedding_model(backoff)?;

    //let (mut markdown, mut code) = swiftide::indexing::Pipeline::from_loader(loader)
    //    .with_concurrency(config.indexing_concurrency())
    //    .with_default_llm_client(indexing_provider)
    //    .filter_cached(storage.clone())
    //    .split_by(|node| {
    //        let Ok(node) = node else { return true };

    //        node.path.extension().is_none_or(|ext| ext == "md")
    //    });

    let pipe = swiftide::indexing::Pipeline::from_stream(statements)
        .with_concurrency(config.indexing_concurrency())
        .with_default_llm_client(indexing_provider)
        .filter_cached(storage.clone());

    //code = code
    //.then_chunk(MultiLanguageChunker::try_for_languages_and_chunk_size(
    //    &config.languages,
    //    CODE_CHUNK_RANGE,
    //)?)
    //.then(transformers::MetadataQACode::default());

    //markdown = markdown
    //    .then_chunk(transformers::ChunkMarkdown::from_chunk_range(
    //        MARKDOWN_CHUNK_RANGE,
    //    ))
    //    .then(transformers::MetadataQAText::default());

    let batch_size = config.indexing_batch_size();
    pipe.then_in_batch(transformers::Embed::new(embedding_provider).with_batch_size(batch_size))
        .then(|mut chunk: Node| {
            chunk
                .metadata
                .insert("path", chunk.path.display().to_string());

            Ok(chunk)
        })
        .then_store_with(storage.clone())
        .run()
        .await?;

    Ok(())
}

async fn garbage_collect(config: &Config) -> Result<()> {
    let garbage_collector = GarbageCollector::from_config(config);
    garbage_collector.clean_up().await
}

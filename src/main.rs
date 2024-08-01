#![feature(allocator_api)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use args::Args;
use clap::Parser as _;
use parser::{Error, Parser};
use tower_lsp::jsonrpc::Result;
use tower_lsp::{lsp_types::*, LanguageServer};
use tower_lsp::{Client, LspService, Server};
use tree::Tree;

mod args;
mod parser;
mod tree;

#[derive(Debug)]
struct Backend {
    client: Client,
    trees: Arc<Mutex<HashMap<Url, Tree>>>,
}

async fn publish_diagnostics(client: &Client, uri: Url, contents: &str, errors: Vec<Error>) {
    let diagnostics = errors
        .iter()
        .map(|error| {
            let start = error.span.start_location(contents).into();
            let end = error.span.end_location(contents).into();

            Diagnostic {
                range: Range { start, end },
                severity: Some(DiagnosticSeverity::ERROR),
                code: None,
                code_description: None,
                source: Some("aoxo-toml".to_string()),
                message: format!("{:?}", error.kind),
                related_information: None,
                tags: None,
                data: None,
            }
        })
        .collect();

    client.publish_diagnostics(uri, diagnostics, None).await
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file changed!")
            .await;

        let contents = params
            .content_changes
            .into_iter()
            .map(|a| a.text)
            .collect::<Vec<_>>()
            .join("");
        let parser = Parser::new(&contents).parse();
        let tree = parser.tree();

        publish_diagnostics(
            &self.client,
            params.text_document.uri.clone(),
            &contents,
            tree.1,
        )
        .await;

        self.trees
            .lock()
            .unwrap()
            .insert(params.text_document.uri, tree.0);
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;

        let contents = params.text_document.text;
        let parser = Parser::new(&contents).parse();
        let tree = parser.tree();

        publish_diagnostics(
            &self.client,
            params.text_document.uri.clone(),
            &contents,
            tree.1,
        )
        .await;

        self.trees
            .lock()
            .unwrap()
            .insert(params.text_document.uri, tree.0);
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Some(file) = args.parse {
        let contents = std::fs::read_to_string(file).unwrap();
        let parser = Parser::new(&contents).parse();
        let tree = parser.tree();
        println!("{:?}", tree);
    } else {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::new(|client| Backend {
            client,
            trees: Arc::default(),
        });
        Server::new(stdin, stdout, socket).serve(service).await;
    }
}

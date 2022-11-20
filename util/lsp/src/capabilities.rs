use tower_lsp::lsp_types::*;

pub fn capabilities() -> lsp::ServerCapabilities {
    let document_symbol_provider = Some(lsp::OneOf::Left(true));

    let text_document_sync = {
        let options = lsp::TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(lsp::TextDocumentSyncKind::FULL),
            ..Default::default()
        };
        Some(lsp::TextDocumentSyncCapability::Options(options))
    };

    let hover_provider = Some(HoverProviderCapability::Simple(true));

    let code_action_provider = Some(lsp::CodeActionProviderCapability::Simple(true));

    lsp::ServerCapabilities {
        text_document_sync,
        document_symbol_provider,
        hover_provider,
        code_action_provider,
        ..Default::default()
    }
}
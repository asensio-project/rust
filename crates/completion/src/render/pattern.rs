//! Renderer for patterns.

use hir::{db::HirDatabase, HasVisibility, Name, StructKind};
use itertools::Itertools;

use crate::{
    config::SnippetCap, item::CompletionKind, render::RenderContext, CompletionItem,
    CompletionItemKind,
};

pub(crate) fn render_struct_pat<'a>(
    ctx: RenderContext<'a>,
    strukt: hir::Struct,
    local_name: Option<Name>,
) -> Option<CompletionItem> {
    let _p = profile::span("render_struct_pat");

    let module = ctx.completion.scope.module()?;
    let fields = strukt.fields(ctx.db());
    let n_fields = fields.len();
    let fields = fields
        .into_iter()
        .filter(|field| field.is_visible_from(ctx.db(), module))
        .collect::<Vec<_>>();

    if fields.is_empty() {
        // Matching a struct without matching its fields is pointless, unlike matching a Variant without its fields
        return None;
    }
    let fields_omitted = n_fields - fields.len() > 0;

    let name = local_name.unwrap_or_else(|| strukt.name(ctx.db())).to_string();
    let mut pat = match strukt.kind(ctx.db()) {
        StructKind::Tuple if ctx.snippet_cap().is_some() => {
            render_tuple_as_pat(&fields, &name, fields_omitted)
        }
        StructKind::Record => {
            render_record_as_pat(ctx.db(), ctx.snippet_cap(), &fields, &name, fields_omitted)
        }
        _ => return None,
    };

    if ctx.completion.is_param {
        pat.push(':');
        pat.push(' ');
        pat.push_str(&name);
    }
    if ctx.snippet_cap().is_some() {
        pat.push_str("$0");
    }

    let mut completion = CompletionItem::new(CompletionKind::Snippet, ctx.source_range(), name)
        .kind(CompletionItemKind::Binding)
        .set_documentation(ctx.docs(strukt))
        .set_deprecated(ctx.is_deprecated(strukt))
        .detail(&pat);
    if let Some(snippet_cap) = ctx.snippet_cap() {
        completion = completion.insert_snippet(snippet_cap, pat);
    } else {
        completion = completion.insert_text(pat);
    }
    Some(completion.build())
}

pub(crate) fn render_variant_pat<'a>(
    ctx: RenderContext<'a>,
    variant: hir::Variant,
    local_name: Option<Name>,
) -> Option<CompletionItem> {
    let _p = profile::span("render_variant_pat");

    let module = ctx.completion.scope.module()?;
    let fields = variant.fields(ctx.db());
    let n_fields = fields.len();
    let fields = fields
        .into_iter()
        .filter(|field| field.is_visible_from(ctx.db(), module))
        .collect::<Vec<_>>();

    let fields_omitted = n_fields - fields.len() > 0;

    let name = local_name.unwrap_or_else(|| variant.name(ctx.db())).to_string();
    let mut pat = match variant.kind(ctx.db()) {
        StructKind::Tuple if ctx.snippet_cap().is_some() => {
            render_tuple_as_pat(&fields, &name, fields_omitted)
        }
        StructKind::Record => {
            render_record_as_pat(ctx.db(), ctx.snippet_cap(), &fields, &name, fields_omitted)
        }
        _ => return None,
    };

    if ctx.completion.is_param {
        pat.push(':');
        pat.push(' ');
        pat.push_str(&name);
    }
    if ctx.snippet_cap().is_some() {
        pat.push_str("$0");
    }
    let mut completion = CompletionItem::new(CompletionKind::Snippet, ctx.source_range(), name)
        .kind(CompletionItemKind::Binding)
        .set_documentation(ctx.docs(variant))
        .set_deprecated(ctx.is_deprecated(variant))
        .detail(&pat);
    if let Some(snippet_cap) = ctx.snippet_cap() {
        completion = completion.insert_snippet(snippet_cap, pat);
    } else {
        completion = completion.insert_text(pat);
    }
    Some(completion.build())
}

fn render_record_as_pat(
    db: &dyn HirDatabase,
    snippet_cap: Option<SnippetCap>,
    fields: &[hir::Field],
    name: &str,
    fields_omitted: bool,
) -> String {
    let fields = fields.iter();
    if snippet_cap.is_some() {
        format!(
            "{name} {{ {}{} }}",
            fields
                .enumerate()
                .map(|(idx, field)| format!("${{{}:{}}}", idx + 1, field.name(db)))
                .format(", "),
            if fields_omitted { ", .." } else { "" },
            name = name
        )
    } else {
        format!(
            "{name} {{ {}{} }}",
            fields.map(|field| field.name(db)).format(", "),
            if fields_omitted { ", .." } else { "" },
            name = name
        )
    }
}

fn render_tuple_as_pat(fields: &[hir::Field], name: &str, fields_omitted: bool) -> String {
    format!(
        "{name}({}{})",
        fields.iter().enumerate().map(|(idx, _)| format!("${}", idx + 1)).format(", "),
        if fields_omitted { ", .." } else { "" },
        name = name
    )
}

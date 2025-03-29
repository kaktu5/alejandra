pub(crate) fn rule(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
) -> std::collections::LinkedList<crate::builder::Step> {
    let mut steps = std::collections::LinkedList::new();

    let mut children = crate::children2::new(build_ctx, node);

    let first = children.next().unwrap();
    let second = children.next().unwrap();

    let vertical = build_ctx.vertical
        || first.has_inline_comment
        || first.has_trivialities
        || second.has_inline_comment
        || second.has_trivialities;

    // first
    if vertical {
        steps.push_back(crate::builder::Step::FormatWider(first.element));
    } else {
        steps.push_back(crate::builder::Step::Format(first.element));
    }

    if let Some(text) = first.inline_comment {
        steps.push_back(crate::builder::Step::Whitespace);
        steps.push_back(crate::builder::Step::Comment(text));
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    }

    let inline_comment = first.trivialities.len() == 1
        && first.trivialities.front().map_or(true, |trivia| {
            matches!(trivia, crate::children2::Trivia::Comment(content) if {
                let is_cstyle = content.starts_with("/*") && content.ends_with("*/");
                let line_count = content.split('\n').count();
                is_cstyle && line_count == 3
            })
        });

    for trivia in first.trivialities {
        match trivia {
            crate::children2::Trivia::Comment(text) if inline_comment => {
                steps.push_back(crate::builder::Step::Whitespace);
                steps.push_back(crate::builder::Step::Comment(text));
            }
            crate::children2::Trivia::Comment(text) => {
                steps.push_back(crate::builder::Step::NewLine);
                steps.push_back(crate::builder::Step::Pad);
                steps.push_back(crate::builder::Step::Comment(text));
            }
            crate::children2::Trivia::Newlines => {}
        }
    }

    // second
    if vertical {
        match (
            (!first.has_inline_comment
                && !first.has_trivialities
                && matches!(
                    second.element.kind(),
                    rnix::SyntaxKind::NODE_ATTR_SET
                        | rnix::SyntaxKind::NODE_LIST
                        | rnix::SyntaxKind::NODE_PAREN
                        | rnix::SyntaxKind::NODE_STRING
                )),
            inline_comment,
        ) {
            (true, false) => {
                steps.push_back(crate::builder::Step::Whitespace);
            }
            (false, true) => {
                steps.push_back(crate::builder::Step::Whitespace);
            }
            _ => {
                steps.push_back(crate::builder::Step::NewLine);
                steps.push_back(crate::builder::Step::Pad);
            }
        }
        steps.push_back(crate::builder::Step::FormatWider(second.element));
    } else {
        steps.push_back(crate::builder::Step::Whitespace);
        steps.push_back(crate::builder::Step::Format(second.element));
    }

    steps
}

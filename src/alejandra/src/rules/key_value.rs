pub(crate) fn rule(
    build_ctx: &crate::builder::BuildCtx,
    node: &rnix::SyntaxNode,
) -> std::collections::LinkedList<crate::builder::Step> {
    let mut steps = std::collections::LinkedList::new();

    let mut children = crate::children::Children::new(build_ctx, node);

    let vertical = build_ctx.vertical
        || children.has_comments()
        || children.has_newlines();

    // a
    let child = children.get_next().unwrap();
    if vertical {
        steps.push_back(crate::builder::Step::FormatWider(child));
    } else {
        steps.push_back(crate::builder::Step::Format(child));
    }

    // /**/
    let mut comment = false;
    children.drain_trivia(|element| match element {
        crate::children::Trivia::Comment(text) => {
            comment = true;
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
            steps.push_back(crate::builder::Step::Comment(text));
        }
        crate::children::Trivia::Whitespace(_) => {}
    });
    if comment {
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    } else {
        steps.push_back(crate::builder::Step::Whitespace);
    }

    // peek: =
    let child_equal = children.get_next().unwrap();

    // peek: /**/
    let mut comments_before = std::collections::LinkedList::new();
    let mut newlines = false;
    children.drain_trivia(|element| match element {
        crate::children::Trivia::Comment(text) => {
            comments_before.push_back(crate::builder::Step::Comment(text))
        }
        crate::children::Trivia::Whitespace(text) => {
            if crate::utils::count_newlines(&text) > 0 {
                newlines = true;
            }
        }
    });

    // peek: expr
    let child_expr = children.get_next().unwrap();

    // Superfluous parens can be removed: `a = (x);` -> `a = x;`
    let child_expr =
        if matches!(child_expr.kind(), rnix::SyntaxKind::NODE_PAREN) {
            let mut children: Vec<rnix::SyntaxElement> =
                child_expr.as_node().unwrap().children_with_tokens().collect();

            if children.len() == 3 {
                children.swap_remove(1)
            } else {
                child_expr
            }
        } else {
            child_expr
        };

    // peek: /**/
    let mut comments_after = std::collections::LinkedList::new();
    children.drain_trivia(|element| match element {
        crate::children::Trivia::Comment(text) => {
            comments_after.push_back(crate::builder::Step::Comment(text))
        }
        crate::children::Trivia::Whitespace(_) => {}
    });

    // =
    let mut dedent = false;
    steps.push_back(crate::builder::Step::Format(child_equal));

    // `key = /*comment*/ value`
    let inline_comment = comments_before.len() == 1
        && comments_before.front().map_or(false, |step| {
            matches!(step, crate::builder::Step::Comment(content) if {
                let is_cstyle = content.starts_with("/*") && content.ends_with("*/");
                let line_count = content.split('\n').count();
                is_cstyle && line_count == 3
            })
        });

    if vertical {
        if inline_comment {
            steps.push_back(crate::builder::Step::Whitespace);
            steps.push_back(comments_before.pop_front().unwrap());
            steps.push_back(crate::builder::Step::Whitespace);
        } else if !comments_before.is_empty() || !comments_after.is_empty() {
            dedent = true;
            steps.push_back(crate::builder::Step::Indent);
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        } else if matches!(
            child_expr.kind(),
            rnix::SyntaxKind::NODE_ASSERT
                | rnix::SyntaxKind::NODE_ATTR_SET
                | rnix::SyntaxKind::NODE_PAREN
                | rnix::SyntaxKind::NODE_LAMBDA
                | rnix::SyntaxKind::NODE_LET_IN
                | rnix::SyntaxKind::NODE_LIST
                | rnix::SyntaxKind::NODE_STRING
                | rnix::SyntaxKind::NODE_WITH
        ) || (matches!(
            child_expr.kind(),
            rnix::SyntaxKind::NODE_APPLY
        )
            && crate::utils::second_through_penultimate_line_are_indented(
                build_ctx,
                child_expr.clone(),
                false,
            ))
        {
            steps.push_back(crate::builder::Step::Whitespace);
        } else {
            dedent = true;
            steps.push_back(crate::builder::Step::Indent);
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
    } else {
        steps.push_back(crate::builder::Step::Whitespace);
    }

    // /**/
    for comment in comments_before {
        steps.push_back(comment);
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    }

    // expr
    if vertical {
        steps.push_back(crate::builder::Step::FormatWider(child_expr));
        if !comments_after.is_empty() {
            steps.push_back(crate::builder::Step::NewLine);
            steps.push_back(crate::builder::Step::Pad);
        }
    } else {
        steps.push_back(crate::builder::Step::Format(child_expr));
    }

    // /**/
    for comment in comments_after {
        steps.push_back(comment);
        steps.push_back(crate::builder::Step::NewLine);
        steps.push_back(crate::builder::Step::Pad);
    }

    // ;
    let child = children.get_next().unwrap();
    steps.push_back(crate::builder::Step::Format(child));
    if dedent {
        steps.push_back(crate::builder::Step::Dedent);
    }

    steps
}

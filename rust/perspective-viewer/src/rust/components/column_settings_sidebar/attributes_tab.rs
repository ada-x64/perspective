// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

mod expression_editor;

use expression_editor::ExprEditorAttr;
use itertools::Itertools;
use yew::{function_component, html, use_callback, Callback, Html, Properties};

use crate::components::viewer::ColumnLocator;
use crate::config::{Expression, ViewConfigUpdate};
use crate::custom_events::CustomEvents;
use crate::model::UpdateAndRender;
use crate::renderer::Renderer;
use crate::session::Session;
use crate::utils::ApiFuture;
use crate::{clone, derive_model, html_template};

#[derive(PartialEq, Clone, Properties)]
pub struct AttributesTabProps {
    pub selected_column: ColumnLocator,
    pub on_close: Callback<()>,
    pub session: Session,
    pub renderer: Renderer,
    pub custom_events: CustomEvents,
}
derive_model!(Session, Renderer for AttributesTabProps);

#[function_component]
pub fn AttributesTab(p: &AttributesTabProps) -> Html {
    let convert_to_expr = {
        clone!(p);
        use_callback((), move |_e, _deps| {
            let task = {
                // 1. Create a new expression.
                let name = p
                    .selected_column
                    .name()
                    .expect("Tried to convert empty expression to expression???");
                let expr_name = format!("\"{name}\""); // will need to be unique
                let expr = Expression::new(&expr_name, &expr_name);
                let mut serde_exprs = p.session.get_view_config().expressions.clone();
                serde_exprs.retain(|expr_name, _expr_val| expr_name != &expr.name);
                serde_exprs.insert(&expr);

                // 2. Replace this column in the view configuration.
                let mut cols = p.session.get_view_config().columns.clone();
                let (idx, _val) = cols
                    .iter()
                    .find_position(|c| c.as_ref().map(|s| s == name).unwrap_or_default())
                    .unwrap_or_else(|| panic!("Couldn't find {name} in view config!"));
                cols[idx] = Some(expr_name.clone());

                // 3. Ensure that the new column is opened, and update
                // p.presentation.set_open_column_settings(Some(expr_name));
                p.update_and_render(ViewConfigUpdate {
                    expressions: Some(serde_exprs),
                    columns: Some(cols),
                    ..Default::default()
                })
            };

            ApiFuture::spawn(task);
        })
    };

    clone!(p.on_close, p.selected_column, p.session, p.renderer);
    html_template! {
        <div id="attributes-tab">
            if matches!(selected_column, ColumnLocator::Expr(_)) {
                <div class="tab-section">
                    <ExprEditorAttr
                        {on_close}
                        {selected_column}
                        {session}
                        {renderer}
                    />
                </div>
            } else {
                <div class="tab-section">
                    <button onclick={convert_to_expr}>
                        {"Convert to expression"}
                    </button>
                </div>
            }
        </div>
    }
}

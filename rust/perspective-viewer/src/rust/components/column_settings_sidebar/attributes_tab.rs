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
use yew::{function_component, html, use_callback, Callback, Html, Properties};

use crate::components::viewer::ColumnLocator;
use crate::custom_events::CustomEvents;
use crate::model::CreateColumn;
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
    let col = p.selected_column.name().cloned().unwrap_or_default();
    let convert_to_expr = {
        clone!(p, col);
        use_callback((), move |_e, _deps| {
            ApiFuture::spawn(p.clone_column(&col, true, true));
        })
    };

    let clone_expr = {
        clone!(p, col);
        use_callback((), move |_e, _deps| {
            ApiFuture::spawn(p.clone_column(&col, false, false))
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
                        {"Convert to Expression Column"}
                    </button>
                </div>
            }
            <div class="tab-section">
                <button onclick={clone_expr}>
                    {"Clone Column"}
                </button>
            </div>
        </div>
    }
}

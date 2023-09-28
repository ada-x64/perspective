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

use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlInputElement;
use yew::{function_component, html, use_state, use_state_eq, Callback, Html, Properties};

use crate::components::expression_editor::{get_new_column_name, ExpressionEditor};
use crate::components::viewer::ColumnLocator;
use crate::config::ViewConfigUpdate;
use crate::model::UpdateAndRender;
use crate::renderer::Renderer;
use crate::session::Session;
use crate::utils::ApiFuture;
use crate::{clone, derive_model, html_template};

#[derive(PartialEq, Clone, Properties)]
pub struct AttributesTabProps {
    pub selected_column: Option<String>,
    pub on_close: Callback<()>,
    pub on_rename_column: Callback<ColumnLocator>,
    pub session: Session,
    pub renderer: Renderer,
}
derive_model!(Renderer, Session for AttributesTabProps);

#[function_component]
pub fn AttributesTab(p: &AttributesTabProps) -> Html {
    tracing::info!("Attributes tab!");
    let is_validating = yew::use_state_eq(|| false);
    let on_save = yew::use_callback(
        |v, p| match &p.selected_column {
            None => save_expr(v, p),
            Some(alias) => update_expr(alias, &v, p),
        },
        p.clone(),
    );

    let on_validate = yew::use_callback(
        |b, validating| {
            validating.set(b);
        },
        is_validating.setter(),
    );

    let on_delete = yew::use_callback(
        |(), p| {
            if let Some(ref s) = p.selected_column {
                delete_expr(s, p);
            }

            p.on_close.emit(());
        },
        p.clone(),
    );

    let default_title = p
        .selected_column
        .clone()
        .unwrap_or_else(|| get_new_column_name(&p.session));
    let title = use_state_eq(|| default_title.clone());

    let on_change_title = {
        clone!(title);
        yew::use_callback(
            move |e: yew::Event, _| {
                let input = e
                    .target()
                    .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
                if let Some(input) = input {
                    title.set(input.value())
                }
            },
            (),
        )
    };

    let on_save_title = {
        clone!(title, default_title);
        yew::use_callback(
            move |_, p| {
                // update column_title
                // update expression alias
                tracing::warn!("on_save_title");
                let title = (*title).clone();
                rename_expr(default_title.clone(), title, p);
            },
            p.clone(),
        )
    };
    let on_reset_title = {
        clone!(title);
        yew::use_callback(
            move |_, _| {
                title.set(default_title.clone());
            },
            (),
        )
    };

    html_template! {
        <div id="attributes-tab">
            <button type="button">{"random button"}</button>
            <div>
                <label class="item_title" for="column-name">{"Column Name"}</label>
                <input onchange={on_change_title} type="text" id="column-name" value={(*title).clone()}/>
                <button type="button" onclick={on_reset_title}>{"Reset"}</button>
                <button type="button" onclick={on_save_title}>{"Save"}</button>
            </div>
            <div class="item_title">{"Expression Editor"}</div>
            <ExpressionEditor
                { on_save }
                { on_validate }
                { on_delete }
                session = { &p.session }
                alias = { p.selected_column.clone() }
            />
        </div>
    }
}

fn rename_expr(old_name: String, new_name: String, props: &AttributesTabProps) {
    let sesh = props.session.clone();
    let exp = sesh.metadata().get_expression_by_alias(&old_name).unwrap();
    clone!(old_name, new_name, props);
    ApiFuture::spawn(async move {
        let update = sesh
            .create_replace_expression_update(
                &old_name,
                &JsValue::from(exp),
                Some(new_name.clone()),
            )
            .await;
        props.update_and_render(update).await?;

        Ok(())
    });
}

fn update_expr(name: &str, new_expression: &JsValue, props: &AttributesTabProps) {
    let n = name.to_string();
    let exp = new_expression.clone();
    let sesh = props.session.clone();
    let props = props.clone();
    ApiFuture::spawn(async move {
        let update = sesh.create_replace_expression_update(&n, &exp, None).await;
        props.update_and_render(update).await?;
        Ok(())
    });
}

fn save_expr(expression: JsValue, props: &AttributesTabProps) {
    tracing::info!("save_expr");
    let task = {
        let expression = expression.as_string().unwrap();
        let mut expressions = props.session.get_view_config().expressions.clone();
        expressions.retain(|x| x != &expression);
        expressions.push(expression);
        props.update_and_render(ViewConfigUpdate {
            expressions: Some(expressions),
            ..Default::default()
        })
    };

    ApiFuture::spawn(task);
}

fn delete_expr(expr_name: &str, props: &AttributesTabProps) {
    let session = &props.session;
    let expression = session
        .metadata()
        .get_expression_by_alias(expr_name)
        .unwrap();

    let mut expressions = session.get_view_config().expressions.clone();
    expressions.retain(|x| x != &expression);
    let config = ViewConfigUpdate {
        expressions: Some(expressions),
        ..ViewConfigUpdate::default()
    };

    let task = props.update_and_render(config);
    ApiFuture::spawn(task);
}

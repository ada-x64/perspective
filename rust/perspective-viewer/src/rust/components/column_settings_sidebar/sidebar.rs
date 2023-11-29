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

use std::fmt::Display;
use std::rc::Rc;

use yew::{function_component, html, Callback, Html, Properties};

use crate::components::column_settings_sidebar::attributes_tab::AttributesTab;
use crate::components::column_settings_sidebar::style_tab::StyleTab;
use crate::components::containers::sidebar::Sidebar;
use crate::components::containers::tablist::{Tab, TabList};
use crate::components::editable_header::EditableHeader;
use crate::components::style::LocalStyle;
use crate::components::type_icon::{TypeIcon, TypeIconType};
use crate::components::viewer::ColumnLocator;
use crate::config::Type;
use crate::custom_events::CustomEvents;
use crate::model::*;
use crate::renderer::Renderer;
use crate::session::Session;
use crate::utils::ApiFuture;
use crate::{clone, css, derive_model, html_template};

#[derive(Debug, Default, Clone, PartialEq, Copy)]
pub enum ColumnSettingsTab {
    #[default]
    Attributes,
    Style,
}

impl Display for ColumnSettingsTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Tab for ColumnSettingsTab {}

#[derive(Clone, Properties)]
pub struct ColumnSettingsProps {
    pub selected_column: ColumnLocator,
    pub on_close: Callback<()>,
    pub session: Session,
    pub renderer: Renderer,
    pub custom_events: CustomEvents,
    pub width_override: Option<i32>,
}

derive_model!(CustomEvents, Session, Renderer for ColumnSettingsProps);

impl PartialEq for ColumnSettingsProps {
    fn eq(&self, other: &Self) -> bool {
        self.selected_column == other.selected_column
    }
}

#[function_component]
pub fn ColumnSettingsSidebar(p: &ColumnSettingsProps) -> Html {
    let get_column_name = {
        clone!(p);
        move || match p.selected_column.clone() {
            ColumnLocator::Expr(Some(name)) | ColumnLocator::Plain(name) => name,
            ColumnLocator::Expr(None) => p.session.metadata().make_new_column_name(None),
        }
    };
    let column_name = yew::use_state_eq(get_column_name.clone());
    {
        clone!(column_name);
        yew::use_effect_with(p.selected_column.clone(), move |_| {
            column_name.set(get_column_name());
        });
    }
    let on_change_column_name = yew::use_callback(column_name.clone(), |s, column_name| {
        column_name.set(s);
    });

    let maybe_ty = p.session.metadata().get_column_view_type(&column_name);

    let expr_contents = yew::use_state(|| {
        Rc::new(
            p.session
                .metadata()
                .get_expression_by_alias(&column_name)
                .unwrap_or_default(),
        )
    });
    let on_expr_input = yew::use_callback(expr_contents.clone(), |val, expr_contents| {
        expr_contents.set(val)
    });

    let header_contents = html! {
        <ColumnSettingsHeader
            {maybe_ty}
            column_name={(*column_name).clone()}
            on_change={on_change_column_name.clone()}
            selected_column={p.selected_column.clone()}
            default_value={(*expr_contents).clone()}
            session={p.session.clone()}
            renderer={p.renderer.clone()}
        />
    };

    let selected_tab = yew::use_state(|| 0);
    let last_clicked_tab = yew::use_state(|| None);
    let on_tab_change = yew::use_callback(
        (selected_tab.clone(), last_clicked_tab.clone()),
        move |(i, tab), (selected_tab, last_clicked_tab)| {
            selected_tab.set(i);
            last_clicked_tab.set(Some(tab));
        },
    );
    {
        clone!(selected_tab);
        yew::use_effect_with(p.selected_column.clone(), move |_| {
            selected_tab.set(0);
        })
    }

    html_template! {
        <LocalStyle href={ css!("column-settings-panel") } />
        <Sidebar
            on_close={p.on_close.clone()}
            id_prefix="column_settings"
            width_override={p.width_override}
            selected_tab={*selected_tab}
            {header_contents}
        >
            <ColumnSettingsTablist
                renderer={p.renderer.clone()}
                session={p.session.clone()}
                custom_events={p.custom_events.clone()}

                on_close={p.on_close.clone()}
                selected_column={p.selected_column.clone()}
                {on_expr_input}

                on_tab_change={on_tab_change.clone()}
                last_clicked_tab={*last_clicked_tab}
                selected_tab={*selected_tab}
                {maybe_ty}
                column_name={(*column_name).clone()}
            />

        </Sidebar>

    }
}

#[derive(PartialEq, Clone, Properties)]
pub struct ColumnSettingsTablistProps {
    renderer: Renderer,
    session: Session,
    custom_events: CustomEvents,

    on_close: Callback<()>,
    selected_column: ColumnLocator,
    on_expr_input: Callback<Rc<String>>,

    on_tab_change: Callback<(usize, ColumnSettingsTab)>,
    selected_tab: usize,
    last_clicked_tab: Option<ColumnSettingsTab>,
    maybe_ty: Option<Type>,
    column_name: String,
}
derive_model!(Renderer, Session, CustomEvents for ColumnSettingsTablistProps);

#[function_component(ColumnSettingsTablist)]
pub fn column_settings_tablist(p: &ColumnSettingsTablistProps) -> Html {
    let mut tabs = vec![];

    let (config, attrs) = (p.get_plugin_config(), p.get_plugin_attrs());
    if config.is_none() || attrs.is_none() {
        tracing::warn!(
            "Could not get full plugin config!\nconfig (plugin.save()): {:?}\nplugin_attrs: {:?}",
            config,
            attrs
        );
    }

    // TODO: This is a hack and needs to be replaced.
    let plugin = p.renderer.get_active_plugin().unwrap();
    let show_styles = p
        .maybe_ty
        .map(|ty| match &*plugin.name() {
            "Datagrid" => ty != Type::Bool,
            "X/Y Scatter" => ty == Type::String,
            _ => false,
        })
        .unwrap_or_default();

    if !matches!(p.selected_column, ColumnLocator::Expr(None))
        && show_styles
        && config.is_some()
        && p.maybe_ty.is_some()
    {
        tabs.push(ColumnSettingsTab::Style);
    }

    if matches!(p.selected_column, ColumnLocator::Expr(_)) {
        tabs.push(ColumnSettingsTab::Attributes);
    }

    let match_fn = yew::use_callback(p.clone(), move |tab, p| match tab {
        ColumnSettingsTab::Attributes => {
            html! {
                <AttributesTab
                    session={ p.session.clone() }
                    renderer={ p.renderer.clone() }
                    custom_events={ p.custom_events.clone() }

                    selected_column={ p.selected_column.clone() }
                    on_close={ p.on_close.clone() }
                    column_name={p.column_name.clone()}
                    on_input={p.on_expr_input.clone()}
                />
            }
        },
        ColumnSettingsTab::Style => html! {
            <StyleTab
                session={ p.session.clone() }
                renderer={ p.renderer.clone() }
                custom_events={ p.custom_events.clone() }

                column_name={ p.column_name.clone() }
                ty={ p.maybe_ty.unwrap() }
            />
        },
    });

    let selected_tab = if p.selected_tab >= tabs.len() {
        0
    } else {
        p.selected_tab
    };

    html! {
        <TabList<ColumnSettingsTab>
            {tabs}
            {match_fn}
            on_tab_change={p.on_tab_change.clone()}
            {selected_tab}
        />
    }
}

#[derive(PartialEq, Properties, Clone)]
pub struct ColumnSettingsHeaderProps {
    maybe_ty: Option<Type>,
    column_name: String,
    on_change: Callback<String>,
    selected_column: ColumnLocator,
    session: Session,
    renderer: Renderer,
    default_value: Rc<String>,
}
derive_model!(Session, Renderer for ColumnSettingsHeaderProps);

#[function_component(ColumnSettingsHeader)]
pub fn column_settings_header(p: &ColumnSettingsHeaderProps) -> Html {
    let header_value_update = yew::use_callback(p.clone(), move |new_name: String, p| {
        if !matches!(p.selected_column, ColumnLocator::Expr(None)) {
            // rename expr
            clone!(p, new_name);
            ApiFuture::spawn(async move {
                let update = p
                    .session
                    .create_rename_expression_update(p.column_name.clone(), new_name.clone())
                    .await;
                p.update_and_render(update).await?;
                Ok(())
            })
        };
        // update currente expr name
        p.on_change.emit(new_name);
    });

    let is_expr = matches!(p.selected_column, ColumnLocator::Expr(_));
    let icon_type = if is_expr {
        TypeIconType::Expr
    } else {
        p.maybe_ty.map(|t| t.into()).unwrap_or(TypeIconType::Expr)
    };
    let header_icon = html! {<TypeIcon ty={icon_type} />};

    html! {
        <EditableHeader
            icon={Some(header_icon)}
            on_value_update={header_value_update}
            editable={is_expr}
            value={p.column_name.clone()}
            default_value={p.default_value.clone()}
        />
    }
}

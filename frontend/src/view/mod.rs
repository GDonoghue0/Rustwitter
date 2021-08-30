#![allow(clippy::wildcard_imports)]
use crate::{Model, Msg, Page, PageData, EventResponse};
use seed::{prelude::*, *};
// use seed::virtual_dom::el_ref::el_ref;
// use shared::responses::UserResponse;
// use web_sys::HtmlInputElement;
// use std::fmt;
use crate::flash::FlashMsg;

// `view` describes what to display.
pub fn view(model: &Model) -> Vec<Node<Msg>> {
    nodes![view_navbar(model), view_page(model), flash(model), ]
}

fn view_navbar(model: &Model) -> Node<Msg> {
    nav![
        C!["navbar", "is-link"],
        attrs!{
            At::from("role") => "navigation",
            At::AriaLabel => "main navigation",
        },
        view_navbar_menu(model),
    ]
}


fn view_navbar_menu(model: &Model) -> Node<Msg> {
    div![
        C!["navbar-menu", IF!(true => "is-active")],
        view_navbar_menu_start(model),
        view_navbar_menu_end(model),
    ]
}

fn view_navbar_menu_start(model: &Model) -> Node<Msg> {
    div![
    C!["navbar-start"],
        if let Some(current_user) = &model.current_user {
            vec![
                a![
                    C!["navbar-item", IF!(matches!(&model.page, Page::Timeline(PageData::NotLoaded) ) => "is-active"),],
                    attrs!{ At::Href => Page::Timeline(PageData::NotLoaded) },
                    "Home",
                ],
                a![
                    C!["navbar-item", IF!(matches!(&model.page, Page::UserProfile(_) ) => "is-active"),],
                    &current_user.username,
                    attrs! { At::Href => Page::UserProfile(current_user.username.clone()) }
                ],
                a![
                    C!["navbar-item", IF!(matches!(&model.page, Page::PostEvent ) => "is-active"),],
                    attrs!{ At::Href => Page::PostEvent },
                    "Post Event",
                ],
            ]
        } else {
            vec![
                a![
                    C!["navbar-item", IF!(matches!(&model.page, Page::Timeline(PageData::NotLoaded) ) => "is-active"),],
                    attrs!{ At::Href => Page::Timeline(PageData::NotLoaded) },
                    "Home",
                ]
            ]
        }
    ]
}

fn view_navbar_menu_end(model: &Model) -> Node<Msg> {
     div![
        C!["navbar-end"],
        div![
            C!["navbar-item"],
            div![
                C!["buttons"],
                if model.logged_in() {
                    view_buttons_for_logged_in_user()
                } else {
                    view_buttons_for_anonymous_user()
                }
            ]
        ]
    ]
}

fn view_buttons_for_logged_in_user() -> Vec<Node<Msg>> {
    vec![
        a![
            C!["button", "is-light"],
            "Log out",
            ev(Ev::Click, |_| Msg::Logout),
        ]
    ]
}

fn view_buttons_for_anonymous_user() -> Vec<Node<Msg>> {
    vec![
        a![
            C!["button", "is-dark"],
            strong!["Sign up"],
            attrs!{ At::Href => Page::SignUp }
        ],
        a![
            C!["button", "is-light"],
            "Log in",
            attrs!{ At::Href => Page::Login },
        ]
    ]
}

fn view_page(model: &Model) -> Node<Msg> {
    match &model.page {
        Page::RootLoggedOut => p!["You're on Root"],
        Page::Login => login(model),
        Page::SignUp => sign_up(model),
        Page::UserProfile(username) => user_profile(username),
        Page::SignedIn => signed_in(),
        Page::Timeline(events) => timeline(model, events),
        Page::PostEvent => post_event(model),
    }
}

fn event(event: &EventResponse) -> Node<Msg> {
    div![
        a![
            "@",
            &event.user.username,
            attrs! {
                At::Href => Page::UserProfile(event.user.username.to_string())
            }
        ],
        br![],
        &event.content,
        br![],
        format!("{:?}", &event.created_at),
        hr![],
    ]
}

fn post_event(model: &Model) -> Node<Msg> {
    div![
        div![input![
            el_ref(&model.post_event_form.text_input),
            attrs! {
                At::Type => "text",
                At::Placeholder => "Whats up?",
            },
        ]],
        div![button![
            "Post",
            ev(Ev::Click, |_| Msg::PostEventFormSubmitted)
        ]]
    ]
}

fn timeline(_model: &Model, events: &PageData<Vec<EventResponse>>) -> Node<Msg> {
    match events {
        PageData::NotLoaded => p!["Loading..."],
        PageData::Loaded(events) => {
            let events_views: Vec<Node<Msg>> = events.iter().map(event).collect::<Vec<_>>();
            div![events_views]
        }
    }
}

fn signed_in() -> Node<Msg> {
    div!["Signed in!"]
}

fn flash(model: &Model) -> Node<Msg> {
    match model.flash.get() {
        None => div![],
        Some(FlashMsg::Notice(msg)) => div!["NOTICE: ", msg],
        Some(FlashMsg::Error(msg)) => div!["ERROR: ", msg],
    }
}

fn login(model: &Model) -> Node<Msg> {
    div![
        div![input![
            el_ref(&model.login_form.username_input),
            attrs! {
                At::Type => "text",
                At::Placeholder => "Username"
            },
        ]],
        div![input![
            el_ref(&model.login_form.password_input),
            attrs! {
                At::Type => "password",
                At::Placeholder => "Password"
            },
        ]],
        div![
        C!["button"],
            button![ev(Ev::Click, |_| Msg::LoginFormSubmitted), "Login", 
            ],
        ],
    ]
}

fn sign_up(model: &Model) -> Node<Msg> {
    div![
        div![input![
            el_ref(&model.sign_up_form.username_input),
            attrs! {
                At::Type => "text",
                At::Placeholder => "Username"
            },
        ]],
        div![input![
            el_ref(&model.sign_up_form.password_input),
            attrs! {
                At::Type => "password",
                At::Placeholder => "Password"
            },
        ]],
        div![
            button![ev(Ev::Click, |_| Msg::SignUpFormSubmitted), "Sign Up"],
        ],
    ]
}


fn user_profile(username: &str) -> Node<Msg> {
    p!["Profile of ", username]
}
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

    nodes![flash(model), nav(model), view_page(model),]

    // let url = Urls::new(&model.base_url);
    //     // div![
    //     //     "Enter address: ", input![
    //     //         el_ref(&model.address),
    //     //         attrs!{At::Placeholder => "Address"},
    //     //     ]
    //     // ],
    //     // div![
    //     //     button![ev(Ev::Click, |_| Msg::CreateUserFormSubmitted), "Submit"],
    //     //     // model
    //     //     //     .user
    //     //     //     .as_ref()
    //     //     //     .map(|user| div![format!("User: {:?}", user)])
    //     // ],
    //     // div![
    //     //     C!["balance"],
    //     //     format!("Address balance: {}", x),
    //     //     // button![ev(Ev::Click, |_| Msg::GetBalance),],
    //     // ]
    //     // div![
    //     //     button![ev(Ev::Click, |_| Msg::Fetch), "Fetch user"],
    //     //     model
    //     //         .user
    //     //         .as_ref()
    //     //         .map(|user| div![format!("User: {:?}", user)])
    //     // ],
    // ]
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

fn nav(model: &Model) -> Node<Msg> {
    if let Some(current_user) = &model.current_user {
        div![
            a!["Home", attrs! { At::Href => Page::Timeline(PageData::NotLoaded) }],
            " | ",
            a!["Post Event", attrs! { At::Href => Page::PostEvent }],
            " | ",
            a![
                &current_user.username,
                attrs! { At::Href => Page::UserProfile(current_user.username.clone()) }
            ],
            " | ",
            a!["Logout", ev(Ev::Click, |_| Msg::Logout), attrs! { At::Href => "" }],
        ]
    } else {
        div![
            a!["Home", attrs! { At::Href => Page::RootLoggedOut }],
            " | ",
            a!["Login", attrs! { At::Href => Page::Login }],
            " | ",
            a!["Sign up", attrs! { At::Href => Page::SignUp }],
        ]
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
            button![ev(Ev::Click, |_| Msg::LoginFormSubmitted), "Login"],
            // model
            //     .user
            //     .as_ref()
            //     .map(|user| div![format!("User: {:?}", user)])
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
            // model
            //     .user
            //     .as_ref()
            //     .map(|user| div![format!("User: {:?}", user)])
        ],
    ]
}


fn user_profile(username: &str) -> Node<Msg> {
    p!["Profile of ", username]
}
// use seed::virtual_dom::el_ref::el_ref;
use seed::{prelude::*, *};
use shared::responses::{PostEventResponse, EventResponse, UserResponse};
use web_sys::HtmlInputElement;
use flash::Flash;
use std::fmt;
// use crate::view::view;

mod view;
mod api;
mod storage;
mod flash;

fn init(url: Url, orders: &mut impl Orders<Msg>) -> Model {
    orders.subscribe(Msg::UrlChanged);
    orders.send_msg(Msg::UrlChanged(subs::UrlChanged(url.clone())));

    let mut model = Model {
        auth_token: storage::get_auth_token(),
        current_user: None,
        page: Page::RootLoggedOut,
        login_form: Default::default(),
        sign_up_form: Default::default(),
        post_event_form: Default::default(),
        flash: Default::default(),
    };

    let page = Page::from(url, &model);
    page.load_data(orders);
    model.page = page;

    if let Some(token) = &model.auth_token {
        orders.perform_cmd(api::reload_current_user(token.to_string()));
    }

    model
}

#[derive(Debug)]
pub struct Model {
    login_form: LoginForm,
    sign_up_form: SignUpForm,
    post_event_form: PostEventForm,
    auth_token: Option<String>,
    current_user: Option<UserResponse>,
    page: Page,
    flash: Flash,
}

impl Model {
    fn set_auth_token(&mut self, token: &str) {
        self.auth_token = Some(token.to_string());
        storage::set_auth_token(token);
    }

    fn remove_auth_token(&mut self) {
        self.auth_token = None;
        self.current_user = None;
        storage::remove_auth_token();
    }

    fn logged_in(&self) -> bool {
        self.auth_token.is_some()
    }
}

#[derive(Debug, Default)]
pub struct LoginForm {
    username_input: ElRef<HtmlInputElement>,
    password_input: ElRef<HtmlInputElement>,
}

#[derive(Debug, Default)]
pub struct SignUpForm {
    username_input: ElRef<HtmlInputElement>,
    password_input: ElRef<HtmlInputElement>,
}

#[derive(Debug, Default)]
pub struct PostEventForm {
    text_input: ElRef<HtmlInputElement>,
}

#[derive(Debug)]
pub enum PageData<T> {
    Loaded(T),
    NotLoaded,
}

#[derive(Debug)]
pub enum Page {
    RootLoggedOut,
    Timeline(PageData<Vec<EventResponse>>),
    Login,
    SignUp,
    UserProfile(String),
    SignedIn,
    PostEvent,
}

impl Page {
    fn go(self, model: &mut Model, orders: &mut impl Orders<Msg>) {
        let url = self
            .to_string()
            .parse::<Url>()
            .expect("Not a URL");

        self.load_data(orders);

        url.go_and_push();
        model.page = self;
    }

    fn load_data(&self, orders: &mut impl Orders<Msg>) {
        match self {
            Page::UserProfile(username) => {
                orders.send_msg(Msg::LoadUserProfile(username.to_string()));
            }
            Page::Timeline(_) => {
                orders.send_msg(Msg::LoadTimeline);
            }
            Page::RootLoggedOut | Page::Login | Page::SignUp | Page::SignedIn | Page::PostEvent => {}
        }
    }

    fn from(url: Url, model: &Model) -> Self {
        let path = url.path().iter().map(|s| s.as_str()).collect::<Vec<_>>();

        match path.as_slice() {
            ["signup"] => Page::SignUp,
            ["login"] => Page::Login,
            ["users", username] => Page::UserProfile(username.to_string()),
            ["signedin"] => Page::SignedIn,
            [] => {
                if model.logged_in() {
                    Page::Timeline(PageData::NotLoaded)
                } else {
                    Page::RootLoggedOut
                }
            }
            ["events", "new"] => Page::PostEvent,
            _ => todo!("Unknown URL: {}", url),
        }
    }
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Page::RootLoggedOut => write!(f, "/"),
            Page::Timeline(_) => write!(f, "/"),
            Page::Login => write!(f, "/login"),
            Page::SignUp => write!(f, "/signup"),
            Page::UserProfile(username) => write!(f, "/users/{}", username.clone()),
            Page::SignedIn => write!(f, "/signedin"),
            Page::PostEvent => write!(f, "/events/new"),
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    LoginFormSubmitted,
    SignUpFormSubmitted,
    CreateUserEndpointResponded(String),
    LoginEndpointResponded(String),
    MeLoaded(UserResponse),
    UrlChanged(subs::UrlChanged),
    LoadUserProfile(String),
    GetUserLoaded(UserResponse),
    EventPosted(EventResponse),
    Error(Error),
    ClearFlash,
    LoadTimelineEndpointResponded(Vec<EventResponse>),
    LoadTimeline,
    PostEventFormSubmitted,
    PostEventEndpointResponded(PostEventResponse),
    Logout,
    Noop
}

#[derive(Debug)]
pub enum Error {
    RequestFailed(FetchError),
}

pub fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    log!("msg recieved, ",  msg);
    match msg {
        Msg::Noop => {}
        Msg::LoginFormSubmitted => {
            let form = &model.login_form;
            let username = form.username_input.get().unwrap().value();
            let password = form.password_input.get().unwrap().value();

            orders.perform_cmd(api::login(username, password));
        }
        Msg::SignUpFormSubmitted => {
            let form = &model.sign_up_form;
            let username = form.username_input.get().unwrap().value();
            let password = form.password_input.get().unwrap().value();

            orders.perform_cmd(api::create_user(username, password));
        }
        Msg::CreateUserEndpointResponded(token) => {
            model.set_auth_token(&token);
            orders.perform_cmd(api::reload_current_user(token.to_string()));
            Page::SignedIn.go(model, orders);
        }
        Msg::LoginEndpointResponded(token) => {
            model.set_auth_token(&token);
            orders.perform_cmd(api::reload_current_user(token.to_string()));
            Page::SignedIn.go(model, orders);
        }
        Msg::MeLoaded(user) => {
            model.current_user = Some(user);
        }
        Msg::UrlChanged(subs::UrlChanged(url)) => {
            log!("url changed to", url.to_string());
            let page = Page::from(url, model);
            page.load_data(orders);
            model.page = page;
        }
        Msg::LoadUserProfile(username) => {
            orders.perform_cmd(api::load_user(username, model.auth_token.clone()));
        }
        Msg::GetUserLoaded(user) => log!("user loaded:", user),
        Msg::EventPosted(event) => log!(event),
        Msg::Error(err) => match err {
            Error::RequestFailed(err) => {
                log!("request failed", err);
                model.flash.set_error("Request Failed", orders);
            }
        }
        Msg::ClearFlash => {
            model.flash.clear()
        }
        Msg::Logout => {
            Page::RootLoggedOut.go(model, orders);
            model.remove_auth_token();
        }
        Msg::LoadTimelineEndpointResponded(events) => {
            if let Page::Timeline(data) = &mut model.page {
                *data = PageData::Loaded(events)
            }
        }
        Msg::LoadTimeline => {
            orders.perform_cmd(api::load_timeline(model.auth_token.clone()));
        }
        Msg::PostEventFormSubmitted => {
            let text = model.post_event_form.text_input.get().unwrap().value();
            orders.perform_cmd(api::post_event(model.auth_token.clone(), text));
        }
        Msg::PostEventEndpointResponded(_) => {
            model.flash.set_notice("Event Posted", orders);
            Page::Timeline(PageData::NotLoaded).go(model, orders);
        }

    }
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view::view);
}

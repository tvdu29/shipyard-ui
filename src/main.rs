use serde::Deserialize;
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::Response,
        fetch::{FetchTask, Request},
        FetchService,
    },
};
use material_yew::{MatList, MatListItem, select::ListIndex};

enum Msg {
    GetList,
    Action(ListIndex, String),
    ReceiveResponse(Result<Repos, anyhow::Error>),
}

#[derive(Deserialize, Debug, Clone)]
struct Repos {
    repositories: Vec<String>,
}

struct RepoList {
    task: Option<FetchTask>,
    list: Option<Repos>,
    link: ComponentLink<Self>,
    error: Option<String>,
}

fn render(image: &String) -> Html {
    html!{<MatListItem>{ image }</MatListItem>}
}

impl RepoList {

    fn view_image_list(&self) -> Html {
        match self.list.clone() {
            Some(ref list) => {
                html! {<MatList onaction= self.link.callback(|val| Msg::Action(val, "basic".to_string()))>
                    { list.repositories.iter().map(|i| render(i)).collect::<Html>() }
                </MatList>}
            }
            None => {
                html! {
                     <button onclick=self.link.callback(|_| Msg::GetList)>
                         { "What are the images ?" }
                     </button>
                }
            }
        }
    }

    fn view_fetching(&self) -> Html {
        if self.task.is_some() {
            html! { <p>{ "Fetching data..." }</p> }
        } else {
            html! { <p></p> }
        }
    }

    fn view_error(&self) -> Html {
        if let Some(ref error) = self.error {
            html! { <p>{ error.clone() }</p> }
        } else {
            html! {}
        }
    }
}

impl Component for RepoList {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            task: None,
            list: None,
            link,
            error: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetList => {
                let request = Request::get("https://docker.adotmob.com/v2/_catalog?n=20")
                    .body(Nothing)
                    .expect("Could not build request");
                let callback =
                    self.link
                        .callback(|response: Response<Json<Result<Repos, anyhow::Error>>>| {
                            let Json(data) = response.into_body();
                            Msg::ReceiveResponse(data)
                        });
                self.task =
                    Some(FetchService::fetch(request, callback).expect("failed to start request"));
                true
            }
            Msg::ReceiveResponse(response) => {
                if let Ok(res) = response {
                    self.list = Some(res);
                    return true;
                };
                false
            }
            _ => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {<>
            {self.view_fetching()}
            {self.view_image_list()}
            {self.view_error()}
        </>}
    }
}

fn main() {
    yew::start_app::<RepoList>();
}

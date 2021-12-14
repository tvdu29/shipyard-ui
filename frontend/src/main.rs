use material_yew::select::ListIndex::Single;
use material_yew::{select::ListIndex, MatList, MatListItem};
use yew::{
    format::{Json, Nothing},
    prelude::*,
    services::{
        fetch::Response,
        fetch::{FetchTask, Request},
        FetchService,
    },
};
use shipyard::{Repos, Tags, DockerManifest};
use anyhow::{self, Error};

enum Msg {
    Error,
    GetList,
    GetImage(String),
    GetManifest(String, String),
    ReceiveResponseTags(Result<Tags, anyhow::Error>),
    ReceiveResponse(Result<Repos, anyhow::Error>),
    ReceiveResponseManifest(Result<DockerManifest, anyhow::Error>)
}

fn render(item: &String) -> Html {
    html! {<MatListItem>{ item }</MatListItem>}
}

fn image_list_callback(val: ListIndex, list: Vec<String>) -> Msg {
    if let Single(index) = val {
        Msg::GetImage(list[index.expect("Error ListIndex type")].clone())
    } else {
        Msg::Error
    }
}

fn image_tags_callback(name: String, val: ListIndex, list: Vec<String>) -> Msg {
    if let Single(index) = val {
        Msg::GetManifest(name, list[index.expect("Error ListIndex type")].clone())
    } else {
        Msg::Error
    }
}

struct Model {
    task: Option<FetchTask>,
    list: Option<Repos>,
    link: ComponentLink<Self>,
    tags: Option<Tags>,
    manifest: Option<DockerManifest>,
    error: Option<Error>,
}

impl Model {
    fn view_tags(&self) -> Html {
        html! {
                match self.tags.clone() {
                Some(ref tags) => {
                    let tags_cp = tags.clone();
                    html! {<MatList onaction= self.link.callback(move |val| image_tags_callback(tags_cp.name.clone(), val, tags_cp.tags.to_vec()))>
                        { tags.tags.iter().map(|i| render(i)).collect::<Html>() }
                    </MatList>}
                }
                None => {
                    html! {
                    }
                }
            }
        }
    }

    fn view_image_list(&self) -> Html {
        match self.list.clone() {
            Some(list) => {
                let list_cp = list.clone();
                html! {<MatList onaction= self.link.callback(move |val|  image_list_callback(val, list_cp.repositories.to_vec()))>
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

    fn view_infos(&self) -> Html {
        match &self.error {
            Some(e) => html!{e},
            None => match self.manifest.clone() {
                Some(man) => match serde_json::to_string_pretty(&man) {
                    Ok(js) => html!{js},
                    Err(_e) => html!{"plop"},
                },
                None => html!{"lol"}
            }
        }
        
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            task: None,
            tags: None,
            link,
            list: None,
            manifest: None,
            error: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GetList => {
                self.list = None;
                self.tags = None;
                self.manifest = None;
                let request = Request::get("http://127.0.0.1:8080/v2/catalog/1")
                    .body(Nothing)
                    .expect("Could not build request");
                println!("callback");
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
            Msg::GetImage(img) => {
                let img = img.replace("/", "%2F");
                self.tags = None;
                self.manifest = None;
                let request =
                    Request::get(format!("http://127.0.0.1:8080/v2/tags/{}", img))
                        .body(Nothing)
                        .expect("Could not build request");
                let callback =
                    self.link
                        .callback(|response: Response<Json<Result<Tags, anyhow::Error>>>| {
                            let Json(data) = response.into_body();
                            Msg::ReceiveResponseTags(data)
                        });
                self.task =
                    Some(FetchService::fetch(request, callback).expect("failed to start request"));
                true
            }
            Msg::GetManifest(img, tag) => {
                let img = img.replace("/", "%2F");
                self.manifest = None;
                let request = Request::get(format!("http://127.0.0.1:8080/v2/manifest/{}:{}", img, tag))
                    .body(Nothing)
                    .expect("Could not build request");
                let callback =
                    self.link
                        .callback(|response: Response<Json<Result<SchemaVersion, anyhow::Error>>>| {
                            let Json(data) = response.into_body();
                            Msg::ReceiveResponseManifest(data)
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
            Msg::ReceiveResponseTags(response) => {
                if let Ok(res) = response {
                    self.tags = Some(res);
                    return true;
                };
                false
            }
            Msg::ReceiveResponseManifest(response) => {
                match response {
                    Ok(res) => {self.manifest = Some(res);
                        return true;},
                    Err(e) => {self.error = Some(e); return true;},
                }
            }
            _ => false,
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="flexWrap">
                <div class="flexCol scroll">{self.view_image_list()}</div>
                <div class="flexCol scroll">{self.view_tags()}</div>
                <div class="flexCol scroll_manifest">{self.view_infos()}</div>
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}

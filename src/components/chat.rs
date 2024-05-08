use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::services::event_bus::EventBus;
use crate::{services::websocket::WebsocketService, User};

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    _producer: Box<dyn Bridge<EventBus>>,
    wss: WebsocketService,
    messages: Vec<MessageData>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://www.svgrepo.com/show/71148/avatar.svg"
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);

        html! {
            <div class="flex w-screen bg-gray-50">
                <div class="flex-none w-64 h-screen bg-teal-100 p-4">
                    <div class="text-2xl font-semibold text-gray-700 mb-4">{"Active Users"}</div>
                    {
                        self.users.clone().iter().map(|u| {
                            html!{
                                <div class="flex items-center m-2 p-2 bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 ease-in-out">
                                    <img class="w-12 h-12 rounded-full border border-teal-500" src={u.avatar.clone()} alt="avatar"/>
                                    <div class="ml-3">
                                        <div class="font-semibold text-teal-600">{u.name.clone()}</div>
                                        <div class="text-sm text-gray-500">{"Online"}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
                <div class="flex flex-col w-full h-screen">
                    <div class="flex items-center justify-between bg-teal-600 text-white px-6 py-3 shadow-lg">
                        <div class="text-xl font-semibold">{"💬 Chat Room"}</div>
                    </div>
                    <div class="flex-grow overflow-auto p-4 bg-white shadow-inner">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from).unwrap();
                                html!{
                                    <div class="flex items-start mb-4">
                                        <img class="w-10 h-10 rounded-full border-2 border-teal-500" src={user.avatar.clone()} alt="avatar"/>
                                        <div class="ml-3 bg-teal-100 p-3 rounded-lg shadow-sm">
                                            <div class="font-bold text-teal-600">{m.from.clone()}</div>
                                            <div class="text-sm text-gray-700 mt-1">
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html!{ <img class="rounded-lg mt-2 shadow-sm" src={m.message.clone()}/> }
                                                    } else {
                                                        html!{ {m.message.clone()} }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                    <div class="flex items-center p-4 bg-teal-50 border-t-2 border-teal-200">
                        <input ref={self.chat_input.clone()} type="text" placeholder="Type your message..." class="flex-grow py-2 px-4 bg-white rounded-full border-2 border-teal-400 outline-none focus:ring-2 focus:ring-teal-600 shadow-sm" required=true />
                        <button onclick={submit} class="ml-4 p-2 bg-teal-600 text-white rounded-full shadow-lg hover:bg-teal-700 transition-colors duration-200 ease-in-out">
                            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 fill-current">
                                <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }    
}
// lib.rs
use yew::prelude::*;
use web_sys::KeyboardEvent;
pub mod game;
pub mod websocket;
pub mod types;
use game::{GameMap, MatchingStatus, GameOver, VirtualKeyboard, styles};
use websocket::WsClient;
use types::{GameMessage, Direction, GameState};

#[function_component(App)]
fn app() -> Html {
    let ws_client = use_state(|| None::<WsClient>);
    let game_state = use_state(|| None::<GameState>);
    let matching_status = use_state(|| (0, 2));
    let game_over_rankings = use_state(|| None::<Vec<(usize, u32)>>);
    let is_ready = use_state(|| false);

    {
        let ws_client = ws_client.clone();
        let game_state_clone = game_state.clone();
        let matching_status_clone = matching_status.clone();
        let game_over_rankings_clone = game_over_rankings.clone();
        
        use_effect_with((), move |_| {
            let mut client = WsClient::new("ws://47.100.220.180:3000/ws");
            
            let game_state_cb = Callback::from(move |state: GameState| {
                game_state_clone.set(Some(state));
            });
            client = client.on_game_state(game_state_cb);
            
            let matching_cb = Callback::from(move |(current, required): (usize, usize)| {
                matching_status_clone.set((current, required));
            });
            client = client.on_matching_status(matching_cb);
            
            let game_over_cb = Callback::from(move |rankings: Vec<(usize, u32)>| {
                game_over_rankings_clone.set(Some(rankings));
            });
            client = client.on_game_over(game_over_cb);
            
            client.start_listening();
            ws_client.set(Some(client));
            || ()
        });
    }

    // 修复：替换 let chain 语法（兼容旧 Rust 版本）+ 修复 move 错误
    {
        let is_ready = is_ready.clone();
        let game_state = game_state.clone();
        use_effect_with((), move |_| {
            let game_state = game_state.clone();
            let is_ready = is_ready.clone();
            // 用普通 if 判断替代 let chain
            if let Some(s) = &*game_state {
                if s.game_started {
                    is_ready.set(true);
                }
            }
            || ()
        });
    }

    let send_message = {
        let ws_client = ws_client.clone();
        move |msg: GameMessage| {
            if let Some(client) = &*ws_client {
                client.send(msg);
            }
        }
    };

    let handle_ready = {
        let send_message = send_message.clone();
        let is_ready = is_ready.clone();
        Callback::from(move |_: MouseEvent| {
            send_message(GameMessage::Ready);
            is_ready.set(true);
        })
    };

    let handle_restart = {
        let send_message = send_message.clone();
        let game_over_rankings = game_over_rankings.clone();
        let is_ready = is_ready.clone();
        Callback::from(move |_: MouseEvent| {
            send_message(GameMessage::Ready);
            game_over_rankings.set(None);
            is_ready.set(true);
        })
    };

    // 处理虚拟键盘方向输入
    let handle_virtual_direction = {
        let send_message = send_message.clone();
        let game_state = game_state.clone();
        Callback::from(move |direction: Direction| {
            let state = game_state.as_ref();
            if state.map_or(true, |s| !s.game_started || s.game_over) {
                return;
            }
            send_message(GameMessage::PlayerInput(direction));
        })
    };

    let handle_keydown = {
        let send_message = send_message.clone();
        let game_state = game_state.clone();
        Callback::from(move |e: KeyboardEvent| {
            let state = game_state.as_ref();
            if state.map_or(true, |s| !s.game_started || s.game_over) {
                return;
            }
            match e.key().as_str() {
                "ArrowUp" => send_message(GameMessage::PlayerInput(Direction::Up)),
                "ArrowDown" => send_message(GameMessage::PlayerInput(Direction::Down)),
                "ArrowLeft" => send_message(GameMessage::PlayerInput(Direction::Left)),
                "ArrowRight" => send_message(GameMessage::PlayerInput(Direction::Right)),
                _ => {}
            }
        })
    };

    // 修复：用嵌套 if 替代 let chain 语法
    let show_matching = {
        let game_state = game_state.clone();
        game_state.as_ref().map_or(false, |s| !s.game_started)
    };

    // 检查游戏是否正在进行（显示虚拟键盘的条件）
    let show_virtual_keyboard = {
        let game_state = game_state.clone();
        game_state.as_ref().map_or(false, |s| s.game_started && !s.game_over)
    };

    html! {
        <div class="app" onkeydown={handle_keydown} tabindex="0" style="outline: none;">
            <h1>{"多人贪吃蛇游戏"}</h1>
            
            if show_matching {
                <MatchingStatus 
                    current={matching_status.0} 
                    required={matching_status.1} 
                    on_ready={handle_ready}
                    is_ready={*is_ready}
                />
            }
            
            <GameMap state={(*game_state).clone()} />
            
            // 添加虚拟键盘（仅在游戏进行中显示）
            if show_virtual_keyboard {
                <VirtualKeyboard on_direction={handle_virtual_direction} />
            }
            
            if let Some(rankings) = &*game_over_rankings {
                <GameOver 
                    rankings={rankings.clone()} 
                    on_restart={handle_restart}
                />
            }

            <style>
                { r#"
                    .app {
                        text-align: center;
                        margin: 20px auto;
                        max-width: 500px;
                        font-family: Arial, sans-serif;
                    }
                    h1 {
                        color: #333;
                        margin-bottom: 30px;
                    }
                "# }
                { styles() }
            </style>
        </div>
    }
}
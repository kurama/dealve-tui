use std::time::Instant;

use dealve_core::models::{Deal, PriceHistoryPoint};
use tokio::task::JoinHandle;

use crate::message::Message;
use crate::model::Model;

pub type DealsLoadTask = JoinHandle<dealve_core::Result<Vec<Deal>>>;
pub type PriceHistoryTask = JoinHandle<(String, dealve_core::Result<Vec<PriceHistoryPoint>>)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadTaskKind {
    StandardDeals,
    SearchDeals,
}

pub struct TaskManager {
    pub load_task: Option<DealsLoadTask>,
    pub load_task_kind: Option<LoadTaskKind>,
    pub load_more_task: Option<DealsLoadTask>,
    pub price_history_task: Option<PriceHistoryTask>,
    pub last_selection_change: Instant,
    pub pending_game_info_load: bool,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            load_task: None,
            load_task_kind: None,
            load_more_task: None,
            price_history_task: None,
            last_selection_change: Instant::now(),
            pending_game_info_load: false,
        }
    }
}

pub fn spawn_deals_load(
    api_key: Option<String>,
    platform_filter: dealve_core::models::Platform,
    region_code: String,
    offset: usize,
    page_size: usize,
    sort: String,
) -> DealsLoadTask {
    tokio::spawn(async move {
        let client = dealve_api::ItadClient::new(api_key);
        let shop_id = platform_filter.shop_id();
        client
            .get_deals(&region_code, page_size, offset, shop_id, Some(&sort))
            .await
    })
}

/// Max results allowed by ITAD search API (`/games/search/v1`).
/// See https://docs.isthereanydeal.com/ — `results: number [1..100]`
const MAX_SEARCH_RESULTS: usize = 100;

pub fn spawn_search_load(
    api_key: Option<String>,
    query: String,
    platform_filter: dealve_core::models::Platform,
    region_code: String,
    limit: usize,
) -> DealsLoadTask {
    tokio::spawn(async move {
        let client = dealve_api::ItadClient::new(api_key);
        client
            .search_deals(
                &query,
                &region_code,
                platform_filter.shop_id(),
                limit.min(MAX_SEARCH_RESULTS),
            )
            .await
    })
}

/// Start the initial/refresh load
pub fn start_load(model: &mut Model, tasks: &mut TaskManager) {
    if let Some(task) = tasks.load_task.take() {
        task.abort();
    }
    tasks.load_task_kind = None;

    if let Some(task) = tasks.load_more_task.take() {
        task.abort();
    }

    model.reset_pagination();
    model.set_loading(true);

    if let Some(query) = model.active_search_query.clone() {
        tasks.load_task_kind = Some(LoadTaskKind::SearchDeals);
        tasks.load_task = Some(spawn_search_load(
            model.api_key.clone(),
            query,
            model.platform_filter,
            model.region.code().to_string(),
            model.deals_page_size,
        ));
    } else {
        tasks.load_task_kind = Some(LoadTaskKind::StandardDeals);
        tasks.load_task = Some(spawn_deals_load(
            model.api_key.clone(),
            model.platform_filter,
            model.region.code().to_string(),
            0,
            model.deals_page_size,
            model.sort_state.api_param(),
        ));
    }
}

/// Check all running tasks and return messages for completed ones
pub async fn check_tasks(model: &mut Model, tasks: &mut TaskManager) -> Vec<Message> {
    let mut messages = Vec::new();

    // Check initial/refresh load
    if let Some(task) = tasks.load_task.as_mut() {
        if task.is_finished() {
            let task = tasks.load_task.take().unwrap();
            let load_kind = tasks
                .load_task_kind
                .take()
                .unwrap_or(LoadTaskKind::StandardDeals);
            let page_size = model.deals_page_size;
            match task.await {
                Ok(Ok(deals)) => {
                    let (is_more, page_size) = match load_kind {
                        LoadTaskKind::StandardDeals => (deals.len() >= page_size, page_size),
                        LoadTaskKind::SearchDeals => (false, deals.len()),
                    };

                    messages.push(Message::DealsLoaded {
                        deals,
                        is_more,
                        page_size,
                    });
                }
                Ok(Err(e)) => {
                    let msg = match load_kind {
                        LoadTaskKind::StandardDeals => e.to_string(),
                        LoadTaskKind::SearchDeals => format!("Search failed: {}", e),
                    };
                    messages.push(Message::DealsLoadFailed(msg));
                }
                Err(_) => {
                    let msg = match load_kind {
                        LoadTaskKind::StandardDeals => "Task failed".to_string(),
                        LoadTaskKind::SearchDeals => "Search task failed".to_string(),
                    };
                    messages.push(Message::DealsLoadFailed(msg));
                }
            }
        }
    }

    // Check load-more task
    if let Some(task) = tasks.load_more_task.as_mut() {
        if task.is_finished() {
            let task = tasks.load_more_task.take().unwrap();
            let page_size = model.deals_page_size;
            match task.await {
                Ok(Ok(deals)) => {
                    let is_more = deals.len() >= page_size;
                    messages.push(Message::MoreDealsLoaded {
                        deals,
                        is_more,
                        page_size,
                    });
                }
                Ok(Err(e)) => {
                    messages.push(Message::DealsLoadFailed(e.to_string()));
                }
                Err(_) => {
                    messages.push(Message::DealsLoadFailed("Task failed".to_string()));
                }
            }
        }
    }

    // Check price history task
    if let Some(task) = tasks.price_history_task.as_mut() {
        if task.is_finished() {
            let task = tasks.price_history_task.take().unwrap();
            if let Ok((game_id, result)) = task.await {
                match result {
                    Ok(history) => {
                        messages.push(Message::PriceHistoryLoaded { game_id, history });
                    }
                    Err(_) => {
                        messages.push(Message::PriceHistoryLoaded {
                            game_id,
                            history: vec![],
                        });
                    }
                }
            }
        }
    }

    // Check if we should load more deals (infinite scroll)
    if !model.is_search_mode()
        && model.should_load_more()
        && tasks.load_more_task.is_none()
        && tasks.load_task.is_none()
    {
        model.pagination.loading_more = true;
        tasks.load_more_task = Some(spawn_deals_load(
            model.api_key.clone(),
            model.platform_filter,
            model.region.code().to_string(),
            model.pagination.offset,
            model.deals_page_size,
            model.sort_state.api_param(),
        ));
    }

    // Check if we should load price history
    if tasks.price_history_task.is_none() && !model.loading.deals {
        if let Some(game_id) = model.needs_price_history_load() {
            model.loading.price_history = Some(game_id.clone());
            let api_key = model.api_key.clone();
            let region_code = model.region.code().to_string();
            tasks.price_history_task = Some(tokio::spawn(async move {
                let client = dealve_api::ItadClient::new(api_key);
                let result = client.get_price_history(&game_id, &region_code).await;
                (game_id, result)
            }));
        }
    }

    messages
}

/// Load game info for the currently selected deal (async, called from main loop)
pub async fn load_game_info_if_needed(model: &mut Model) {
    if let Some(game_id) = model.needs_game_info_load() {
        model.loading.game_info = Some(game_id.clone());
        let client = dealve_api::ItadClient::new(model.api_key.clone());
        if let Ok(info) = client.get_game_info(&game_id).await {
            model.game_info_cache.insert(game_id.clone(), info);
        }
        if model.loading.game_info.as_ref() == Some(&game_id) {
            model.loading.game_info = None;
        }
    }
}

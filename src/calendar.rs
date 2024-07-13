use google_calendar3::chrono::DateTime;
use google_calendar3::{CalendarHub, oauth2, hyper, hyper_rustls, chrono};

use log::info;
use std::time::Instant;

#[derive(Debug)]
pub struct CalendarEvent {
    pub start_time: DateTime<chrono::Utc>,
    pub name: String,
}

pub async fn fetch_event() -> Result<Option<CalendarEvent>, String> {

    info!("Fetching calendar...");
    let now = Instant::now();

    /* Loading user token */
    let secret = oauth2::read_application_secret("sensitive/creds.json")
        .await
        .map_err(|e| format!("Failed to read application secret: {}", e))?;

    /* Loading "kindle app" authentication */
    let auth = oauth2::InstalledFlowAuthenticator::builder(secret, oauth2::InstalledFlowReturnMethod::HTTPRedirect)
        .persist_tokens_to_disk("sensitive/tokencache.json")
        .build()
        .await
        .map_err(|e| format!("Failed to build authenticator: {}", e))?;

    /* Main struct for interating with google api */
    let hub = CalendarHub::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .map_err(|e| format!("Failed to build calendar hub: {}", e))?
            .https_or_http()
            .enable_http1()
            .build()
        ),
        auth,
    );

    let mut found_events: Vec<CalendarEvent> = Vec::new();

    /* Finding all user calenders */
    let (_, calendar_list) = hub.calendar_list().list().doit().await
        .map_err(|e| format!("Could not load calendars: {}", e))?;

    if let Some(calendars) = calendar_list.items {

        /* For every found user calender */
        
        for calendar in calendars {
            if let Some(mut cid) = calendar.id {

                /* Clean calendar id (perm error otherwise) */
                cid = cid.replace("#", "%23");

                /* Getting the calendar events */
                let (_, events) = hub.events().list(&cid)
                    .time_min(chrono::Utc::now())
                    .add_scope("https://www.googleapis.com/auth/calendar")
                    .doit().await
                    .map_err(|e| format!("Failed to list events for calendar {}: {}", cid, e))?;
                
                /* Adding the found events to the results vector */
                if let Some(items) = events.items {
                    for event in items {

                        /* Start time & event summary not guaranteed */
                        if let Some(start) = event.start {
                            if let Some(start_time) = start.date_time {
                                
                                /* Add to result vector */
                                found_events.push(CalendarEvent {
                                    start_time,
                                    name: event.summary.unwrap_or_else(|| "No Title".to_string()),
                                });
                            }
                        }
                    }
                }

                // Ignore any items we weren't able to fetch - more than likely a perm. err. 
            }
        }
    }

    /* Sort by closest events to now (we only collect events after now)*/
    found_events.sort_by_key(|d| d.start_time);

    let elapsed = format!("{:.2?}", now.elapsed());
    info!("Calendar took {elapsed}");

    /* Return the next event, if one exists */
    let next_event = found_events.pop();
    if let Some(event) = next_event {
        return Ok(Some(event))
    }

    Ok(None)
}

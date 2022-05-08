
use std::error::Error;
use twitch_gql::{TwitchGqlClient, GqlRequestBuilder, ClipsFullVideoButton, ApiResponse};

#[tokio::test]
async fn test_clip_endpoint() -> Result<(), Box<dyn Error>> {

    let client = TwitchGqlClient::new_unauth("kimne78kx3ncx6brgo4mv6wki5h1ko");
    let request = GqlRequestBuilder::new()
        .clips_full_video_button("SquareProtectiveSpindleTBTacoLeft-0NH8oKbYyIg-SqA0");
    let res = client.send_request(request).await?;
    let r = serde_json::from_value::<Vec<ApiResponse<ClipsFullVideoButton>>>(res)?;

    println!("{:?}", r);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].data.clip.video_offset_seconds, 5672);
    Ok(())
}
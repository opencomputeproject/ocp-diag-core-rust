// (c) Meta Platforms, Inc. and affiliates.
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use anyhow::Result;
use serde_json::json;

use ocptv::output::{File, Uri};

use super::fixture::*;

#[tokio::test]
async fn test_step_with_file() -> Result<()> {
    let uri = Uri::parse("file:///tmp/foo")?;
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "file": {
                    "name": "name",
                    "uri": uri.clone().as_str().to_owned(),
                    "isSnapshot": false
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(4),
        json_run_pass(5),
    ];

    check_output_step(&expected, |s, _| async move {
        s.add_file("name", uri).await?;

        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_step_with_file_builder() -> Result<()> {
    let uri = Uri::parse("file:///tmp/foo")?;
    let expected = [
        json_schema_version(),
        json_run_default_start(),
        json_step_default_start(),
        json!({
            "testStepArtifact": {
                "testStepId": "step0",
                "file": {
                    "name": "name",
                    "uri": uri.clone().as_str().to_owned(),
                    "isSnapshot": false,
                    "contentType": "text/plain",
                    "description": "description",
                    "metadata": {
                        "key": "value"
                    },
                }
            },
            "sequenceNumber": 3,
            "timestamp": DATETIME_FORMATTED
        }),
        json_step_complete(4),
        json_run_pass(5),
    ];

    check_output_step(&expected, |s, _| async move {
        let file = File::builder("name", uri)
            .content_type(mime::TEXT_PLAIN)
            .description("description")
            .add_metadata("key", "value".into())
            .build();
        s.add_file_detail(file).await?;

        Ok(())
    })
    .await
}

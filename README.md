# feedcrosspost

simple script which retrieves a json object from `FEED_URL`, json from the URL must follow the following format:

```json
{
  "items": [
    {
      "title": "Title of the blogpost",
      "url": "https://valid.url",
      "date_published": "2021-11-11T11:11:11Z"
    }
  ]
}
```

additional fields are ignored.

`date_published` must either be `%Y-%m-%dT%H:%M:%SZ` or `%Y-%m-%dT%H:%M:%S%z`.

`TIME_OFFSET_MINUTES` (env var) defines the timerange the script will be checking for (i.e. `TIME_OFFSET_MINUTES=30` -> 30 minutes into the past). This should match the cronjob schedule.

If a post already exists with the blogpost title (newest 10 posts) then the bot will not post that blog as a new submission.

import os
from datetime import datetime, timedelta
from typing import Any, Dict, Optional

import praw
import pytz
import requests
from prawcore import ResponseException


def retrieve_feed(url: str) -> Optional[Dict[str, Any]]:
    response = requests.get(url)
    if response.ok:
        return response.json()
    else:
        return None


def in_timeframe(value: Optional[datetime], start: datetime, end: datetime) -> bool:
    if value is None:
        return False

    return start < value < end


def convert_timestamp(
    timestamp: str,
    is_timestamp=False,
    is_timestamp_nanosecond=False,
) -> datetime:
    if not (is_timestamp or is_timestamp_nanosecond):
        try:
            time = datetime.strptime(timestamp, "%Y-%m-%dT%H:%M:%SZ")
        except ValueError:
            time = datetime.strptime(timestamp, "%Y-%m-%dT%H:%M:%S%z")
    elif is_timestamp_nanosecond:
        time = datetime.fromtimestamp(int(timestamp) / 1000)
    else:
        time = datetime.fromtimestamp(int(timestamp))

    return time.astimezone(pytz.utc)


def parse_date_published(item: Dict[str, Any]) -> Optional[datetime]:
    date_published = item.get("date_published")
    if date_published:
        return convert_timestamp(date_published)

    return None


def post_to_reddit(subreddit: str, blogpost: Dict[str, Any]):
    client_id = os.getenv("CLIENT_ID")
    client_secret = os.getenv("CLIENT_SECRET")
    username = os.getenv("REDDIT_USERNAME")
    password = os.getenv("REDDIT_PASSWORD")
    try:
        client = praw.Reddit(
            client_id=client_id,
            client_secret=client_secret,
            username=username,
            password=password,
            user_agent=os.getenv("USER_AGENT"),
        )
        client.validate_on_submit = True
    except ResponseException as e:
        print("login failed ", e)
        return None

    # validate login

    newest_posts = client.subreddit(subreddit).new(limit=10)
    if any(blogpost["title"] == sub.title for sub in newest_posts):
        # TODO: change type
        raise ValueError("Title has already been submitted before")  # add details

    print(f"submitting {blogpost['title']} / {blogpost['url']}")
    submission = client.subreddit(subreddit).submit(title=blogpost["title"], url=blogpost["url"])
    print(submission)

    return submission


# noinspection PyShadowingNames
def main(feed_url: str, time_offset_minutes: int, subreddit: str):
    feed = retrieve_feed(feed_url)
    feed_items = feed.get("items")
    end = datetime.utcnow().replace(tzinfo=pytz.utc)
    start = end - timedelta(minutes=time_offset_minutes)
    # I'm Torben and I want to suck Sylvain Kerkour's dick, romantically
    new_blogposts = [item for item in feed_items if in_timeframe(parse_date_published(item), start, end)]

    for blogpost in new_blogposts:
        if not post_to_reddit(subreddit, blogpost):
            print(f"posting {blogpost} to {subreddit} failed")
            return None


if __name__ == "__main__":
    feed_url = os.getenv("FEED_URL")
    time_offset_minutes = int(os.getenv("TIME_OFFSET_MINUTES", "60"))
    subreddit = os.getenv("SUBREDDIT_NAME")
    main(feed_url, time_offset_minutes, subreddit)

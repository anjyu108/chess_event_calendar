import datetime
import os.path
import argparse

from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from googleapiclient.errors import HttpError
from google.auth.exceptions import RefreshError


# TODO: introduce logger


# If modifying these scopes, delete the file token.json.
SCOPES = ["https://www.googleapis.com/auth/calendar.events"]
# SCOPES = ["https://www.googleapis.com/auth/calendar"]

# calendar ID of: Japan Chess Event
CALENDAR_ID = "07de3fa594f35d7d04199155ec8ca7089e3053f3e2bbd305a51bf8f1cdcee2d9@group.calendar.google.com"


def authenticate(cred_path):
    creds = None
    token_cache_path = "token.json"  # TODO: parametarize

    print("Authenticate")

    if os.path.exists(token_cache_path):
        creds = Credentials.from_authorized_user_file(token_cache_path, SCOPES)

    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            flow = InstalledAppFlow.from_client_secrets_file(
                'credentials.json', SCOPES)
            creds = flow.run_local_server(port=0)
        # Save the credentials for the next run
        with open(token_cache_path, 'w') as token:
            token.write(creds.to_json())
    return creds


def main(args):
    """Shows basic usage of the Google Calendar API.
    Prints the start and name of the next 10 events on the user's calendar.
    """
    if not os.path.exists(args.cred_path):
        print(f"--cred_path not exit: {args.cred_path}")
        return

    creds = authenticate(args.cred_path)

    try:
        service = build("calendar", "v3", credentials=creds)

        print("Insert test event")
        start_time = datetime.datetime(2024, 8, 15, 9, 0, 0)
        end_time = datetime.datetime(2024, 8, 15, 18, 0, 0)
        event = {
          'summary': 'Test Event',
          'location': 'test location',
          'description': 'my location',
          'start': {
            'dateTime': start_time.strftime('%Y-%m-%dT%H:%M:%S'),
            'timeZone': 'Asia/Tokyo',
          },
          'end': {
            'dateTime': end_time.strftime('%Y-%m-%dT%H:%M:%S'),
            'timeZone': 'Asia/Tokyo',
          },
          'description': "test description",
          'colorId': "2",
        }
        event = service.events().insert(calendarId=CALENDAR_ID,
                                        body=event) \
                                .execute()


        print("List the upcomming 10 events")
        now = datetime.datetime.utcnow().isoformat() + "Z"  # 'Z' indicates UTC time
        events_result = (
            service.events()
            .list(
                calendarId=CALENDAR_ID,
                timeMin=now,
                maxResults=10,
                singleEvents=True,
                orderBy="startTime",
            )
            .execute()
        )
        events = events_result.get("items", [])
        if not events:
            print("No upcoming events found.")
            return
        for event in events:
            start = event["start"].get("dateTime", event["start"].get("date"))
            print(start, event["summary"])

    except HttpError as error:
        print(f"An error occurred: {error}")
    except RefreshError as error:
        print(f"An error occurred: {error}")
        print(f"Consider to delete token.json to referesh it")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--cred_path", default="credentials.json")
    args = parser.parse_args()

    main(args)

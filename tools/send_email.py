import json
import smtplib
import os
import ssl
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from dotenv import load_dotenv

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
DATA_DIR = os.path.join(SCRIPT_DIR, "..", "data")
MESSAGES_PATH = os.path.join(DATA_DIR, "messages.json")
# Messages currently being sent. Kept separate from messages.json so the
# server can keep appending to a fresh file while this run works through a
# snapshot, instead of racing a read-then-delete against it.
PROCESSING_PATH = os.path.join(DATA_DIR, "messages.processing.json")

env_path = os.path.join(SCRIPT_DIR, "..", "secrets", "mail.env")
load_dotenv(env_path)

sender_email = os.getenv("SENDER_EMAIL")
sender_password = os.getenv("SENDER_PASSWORD")
receiver_email = "mechardo.labs@gmail.com"
smtp_server = "smtp.gmail.com"
smtp_port = 587  # TLS port (recommended for Gmail)


def claim_pending_messages():
    """Atomically move messages.json out of the way before reading it.

    A leftover messages.processing.json means the previous run crashed
    mid-send; pick that back up instead of claiming messages.json again, so
    a message the server is about to write there isn't skipped.
    """
    if os.path.exists(PROCESSING_PATH):
        return
    try:
        os.rename(MESSAGES_PATH, PROCESSING_PATH)
    except FileNotFoundError:
        pass


def load_pending_messages():
    try:
        with open(PROCESSING_PATH, "r") as file:
            return json.load(file)
    except FileNotFoundError:
        return []


def save_pending_messages(messages):
    # Once every message has been sent, drop the file instead of leaving an
    # empty array around - its absence is what lets the next run tell "still
    # working through a batch" apart from "nothing to do".
    if not messages:
        os.remove(PROCESSING_PATH)
        return
    with open(PROCESSING_PATH, "w") as file:
        json.dump(messages, file, indent=2)


def send_message(server, entry):
    email = entry["email"]
    message = entry["message"]
    name = entry["name"]
    timestamp = entry["timestamp"]

    subject = f"Message from {name}"
    body = f"""
    From: {name} ({email})
    Timestamp: {timestamp}
    Message: {message}
    """

    msg = MIMEMultipart()
    msg["From"] = sender_email
    msg["To"] = receiver_email
    msg["Reply-To"] = email
    msg["Subject"] = subject
    msg.attach(MIMEText(body, "plain"))

    server.sendmail(sender_email, receiver_email, msg.as_string())
    print(f"Email sent for {name} ({email})")


def main():
    claim_pending_messages()
    pending = load_pending_messages()
    if not pending:
        # Normal state on most runs (this executes every minute from cron),
        # so stay quiet instead of growing the log forever.
        return

    if not sender_email or not sender_password:
        raise ValueError(f"SENDER_EMAIL or SENDER_PASSWORD not found in {env_path}")

    try:
        server = smtplib.SMTP(smtp_server, smtp_port)
        server.set_debuglevel(0)  # Set to 1 to dump the SMTP dialogue when debugging
        server.starttls(context=ssl.create_default_context())  # Enable TLS with secure context
        server.login(sender_email, sender_password)
    except smtplib.SMTPAuthenticationError as auth_err:
        print(f"SMTP Authentication Error: {auth_err.smtp_code} {auth_err.smtp_error.decode()}")
        print("Possible causes:")
        print(f"- Incorrect email or password in {env_path}.")
        print("- 2-Step Verification enabled: Use an App Password instead of your regular password.")
        print("- Less Secure App Access disabled: Enable it or use an App Password.")
        print("- Google blocked the login attempt: Check your Google Account for security alerts.")
        print("See https://myaccount.google.com/security for App Password setup.")
        raise
    except smtplib.SMTPException as smtp_err:
        print(f"SMTP Error: {str(smtp_err)}")
        print("Check SMTP server (smtp.gmail.com) and port (587).")
        raise

    try:
        # Sent messages are dropped from `pending` and the file rewritten
        # right away, so a failure partway through never resends the ones
        # that already went out on the next run.
        while pending:
            send_message(server, pending[0])
            pending.pop(0)
            save_pending_messages(pending)
    finally:
        server.quit()


if __name__ == "__main__":
    try:
        main()
    except ValueError as ve:
        print(f"Error: {str(ve)}")
    except Exception as e:
        print(f"An unexpected error occurred: {str(e)}")

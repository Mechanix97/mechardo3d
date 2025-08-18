import json
import smtplib
import os
import ssl
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart
from dotenv import load_dotenv

# Load environment variables from ../secrets/mail.env
env_path = os.path.join(os.path.dirname(__file__), '..', 'secrets', 'mail.env')
load_dotenv(env_path)

# Email configuration from .env
sender_email = os.getenv("SENDER_EMAIL")
sender_password = os.getenv("SENDER_PASSWORD")
receiver_email = "mechardo.labs@gmail.com"
smtp_server = "smtp.gmail.com"
smtp_port = 587  # TLS port (recommended for Gmail)

# File path for the JSON file
file_path = "../data/messages.json"

# Read the JSON file
try:
    with open(file_path, "r") as file:
        data = json.load(file)

    # Validate environment variables
    if not sender_email or not sender_password:
        raise ValueError("SENDER_EMAIL or SENDER_PASSWORD not found in ../secrets/mail.env")

    # Set up the SMTP server
    try:
        server = smtplib.SMTP(smtp_server, smtp_port)
        server.set_debuglevel(1)  # Enable debug output for troubleshooting
        server.starttls(context=ssl.create_default_context())  # Enable TLS with secure context
        server.login(sender_email, sender_password)
    except smtplib.SMTPAuthenticationError as auth_err:
        print(f"SMTP Authentication Error: {auth_err.smtp_code} {auth_err.smtp_error.decode()}")
        print("Possible causes:")
        print("- Incorrect email or password in ../secrets/mail.env.")
        print("- 2-Step Verification enabled: Use an App Password instead of your regular password.")
        print("- Less Secure App Access disabled: Enable it or use an App Password.")
        print("- Google blocked the login attempt: Check your Google Account for security alerts.")
        print("See https://myaccount.google.com/security for App Password setup.")
        raise
    except smtplib.SMTPException as smtp_err:
        print(f"SMTP Error: {str(smtp_err)}")
        print("Check SMTP server (smtp.gmail.com) and port (587).")
        raise

    # Iterate through each entry in the JSON data
    for entry in data:
        email = entry["email"]
        message = entry["message"]
        name = entry["name"]
        timestamp = entry["timestamp"]

        # Create the email content
        subject = f"Message from {name}"
        body = f"""
        From: {name} ({email})
        Timestamp: {timestamp}
        Message: {message}
        """

        # Set up the MIME
        msg = MIMEMultipart()
        msg["From"] = sender_email
        msg["To"] = receiver_email
        msg["Subject"] = subject
        msg.attach(MIMEText(body, "plain"))

        # Send the email
        server.sendmail(sender_email, receiver_email, msg.as_string())
        print(f"Email sent for {name} ({email})")

    # Close the SMTP server connection
    server.quit()

    # Delete the file after sending emails
    os.remove(file_path)
    print(f"File {file_path} has been deleted.")

except FileNotFoundError as e:
    if str(e).startswith("[Errno 2] No such file or directory:") and 'mail.env' in str(e):
        print(f"Error: The file ../secrets/mail.env was not found.")
    else:
        print(f"Error: The file {file_path} was not found.")
except ValueError as ve:
    print(f"Error: {str(ve)}")
except Exception as e:
    print(f"An unexpected error occurred: {str(e)}")

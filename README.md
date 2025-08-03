# Usage

To use it with b4/git-send-email, simply activate it in your git config:

```
git config set --global sendemail.smtpserver /usr/bin/msmtp-retry
git config set --global sendemail.b4-really-reflect-via /usr/bin/msmtp-retry
```

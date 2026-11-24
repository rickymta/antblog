# LNX-029 · Xác thực hai bước cho SSH bằng mã OTP
#clear
## Bí mật chung của mã OTP
#file ~/.google_authenticator google_authenticator-an
chmod 400 ~/.google_authenticator
ls -l ~/.google_authenticator
oathtool --totp -b "$(head -n 1 ~/.google_authenticator)" | wc -c
#shot 06
#clear
## PAM: thay mật khẩu Unix bằng mã OTP
#sudofile sudoedit /etc/pam.d/sshd pam-sshd 01
grep -n -E 'google|permit|common-auth' /etc/pam.d/sshd
#shot 07
## sshd: cần cả key lẫn mã OTP
#sudofile sudoedit /etc/ssh/sshd_config.d/05-an-toan.conf sshd-an-toan-mfa 02
sudo sshd -t && echo "cú pháp ổn"
sudo sshd -T | grep -E '^(usepam|kbdinteractiveauthentication|authenticationmethods) '
sudo systemctl reload ssh
#shot 08
#gui dang-nhap-mfa
#clear
## Bằng chứng trong log
#login an an-laptop
#login an an-laptop otp
sudo journalctl -u ssh --no-pager | grep -E 'Accepted|Partial|Failed|Connection closed by authenticating' | tail -n 4
#shot 05

# LNX-028 · SSH agent, passphrase và nhiều key
#clear
## Một key có passphrase trên server
ssh-keygen -q -t ed25519 -C an@toinoi-app-01 -N mot-cum-tu-chi-dung-cho-lab -f ~/.ssh/id_ed25519
ls -l ~/.ssh
head -n 1 ~/.ssh/id_ed25519
ssh-keygen -y -P sai-passphrase -f ~/.ssh/id_ed25519
cat ~/.ssh/id_ed25519.pub >> ~/.ssh/authorized_keys
ssh -o BatchMode=yes localhost hostname
#shot 01
#clear
## ssh-agent giữ key đã mở khoá
eval "$(ssh-agent -s)"
#root install -D -m 755 /usr/share/toinoi-lab/p03/askpass-passphrase /usr/local/libexec/lab-askpass
#@ export SSH_ASKPASS=/usr/local/libexec/lab-askpass SSH_ASKPASS_REQUIRE=force
ssh-add ~/.ssh/id_ed25519
#gui nhap-passphrase
#@ unset SSH_ASKPASS SSH_ASKPASS_REQUIRE
ssh-add -l
ssh -o BatchMode=yes localhost hostname
## ~/.ssh/config: tên ngắn và key riêng cho từng server
#file ~/.ssh/config ssh-config
chmod 600 ~/.ssh/config
ssh -G app01 | grep -E '^(hostname|user|port|identityfile|identitiesonly) '
ssh app01 hostname
#shot 02
#clear
## Gỡ key khỏi agent, tắt agent
ssh-add -D
ssh-add -l
ssh-agent -k
#root rm -f /usr/local/libexec/lab-askpass
#gui agent-ssh-studio

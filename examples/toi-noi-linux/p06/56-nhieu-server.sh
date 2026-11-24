# LNX-056 · Chạy một lệnh trên nhiều server: vòng lặp SSH và Ansible tối thiểu
#clear
## Vòng lặp SSH
#file ~/.ssh/config ssh-config
ssh toinoi-bastion hostname
for h in toinoi-app-02 toinoi-bastion; do echo "== $h"; ssh $h 'uptime; df -h / | tail -n 1'; done
for h in toinoi-app-02 toinoi-bastion; do ssh $h 'grep PRETTY /etc/os-release' | sed "s/^/$h: /"; done
for h in toinoi-app-02 toinoi-khong-co; do ssh -o ConnectTimeout=3 $h hostname || echo "LOI: $h"; done
#shot 01
#clear
## Ansible tối thiểu
sudo apt install -y ansible-core 2>&1 | grep -E '^Setting up ansible-core'
ansible --version | head -n 1
mkdir -p ~/van-hanh && cd ~/van-hanh
#file hosts.ini hosts.ini
ansible all -i hosts.ini -m ping
ansible all -i hosts.ini -a 'uptime'
#shot 02
#clear
## Playbook: cùng một cấu hình cho mọi server
#file ssh-toinoi.yml ssh-toinoi.yml 03
mkdir -p ~/.config
#file ~/.config/ansible-become ansible-become
chmod 600 ~/.config/ansible-become
ansible-playbook -i hosts.ini ssh-toinoi.yml --become-password-file ~/.config/ansible-become
ansible-playbook -i hosts.ini ssh-toinoi.yml --become-password-file ~/.config/ansible-become | tail -n 3
ansible all -i hosts.ini -a 'sshd -T' --become --become-password-file ~/.config/ansible-become | grep -E 'CHANGED|SUCCESS|maxauthtries'
#shot 04
cd ~

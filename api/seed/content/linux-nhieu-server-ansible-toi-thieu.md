---
slug: linux-nhieu-server-ansible-toi-thieu
title: "Chạy một lệnh trên nhiều server: vòng lặp SSH và Ansible tối thiểu"
summary: "Từ toinoi-app-01: chạy cùng một lệnh trên app02 và bastion bằng vòng lặp ssh dựa trên ~/.ssh/config, xử lý máy không tới được, rồi cài ansible-core, viết inventory, chạy lệnh ad-hoc, và một playbook áp cùng chính sách SSH cho mọi server với validate, handler và tính lặp lại (chạy lần hai không đổi gì)."
category: devops-linux
level: intermediate
tags: [linux, lnx-056, ssh, ansible, automation, playbook, ssh-studio, toi-noi]
language: vi
---

Bình nhờ An kiểm tra nhanh: ba server đang chạy bản Ubuntu nào, đĩa còn bao nhiêu. Rồi thêm một việc: đặt `MaxAuthTries 3` cho sshd trên mọi máy, như đã làm trên app01. Mở từng tab, gõ từng lệnh, sửa từng file thì được, với ba máy. Với ba mươi máy thì không. SSH Studio không có tính năng gõ một lần cho nhiều phiên, và cũng không cần: việc này thuộc về dòng lệnh, từ một máy trong mạng nội bộ.

<div class="tkh-callout">
<span class="tkh-callout-title">LNX-056 · Phần 6/6 · Trung bình</span>
<p>Đọc trước: <a href="/articles/linux-ssh-agent-passphrase">LNX-028</a> (<code>~/.ssh/config</code>), <a href="/articles/linux-dong-bo-rsync-ssh-studio">LNX-046</a> (key đồng bộ), <a href="/articles/linux-bash-dieu-kien-vong-lap-ham">LNX-017</a> (vòng lặp). Trạng thái đầu bài: <code>npm run setup -- 56</code>. Key đồng bộ của app01 đã được Bình cài lên bastion. Bài cài <code>ansible-core</code> từ kho của Ubuntu (cần Internet).</p>
</div>

## 1. Vòng lặp SSH

Thêm bastion vào `~/.ssh/config` của An trên app01. app01 ở trong mạng nội bộ nên đi thẳng, không cần jump host:

```text
# Bastion: app01 nằm trong mạng nội bộ nên đi thẳng, cùng key đồng bộ (LNX-056).
Host toinoi-bastion bastion
    HostName toinoi-bastion
    User an
    IdentityFile ~/.ssh/dong-bo
    IdentitiesOnly yes
    StrictHostKeyChecking accept-new
```

```text
an@toinoi-app-01:~$ ssh toinoi-bastion hostname
Warning: Permanently added 'toinoi-bastion' (ED25519) to the list of known hosts.
toinoi-bastion
an@toinoi-app-01:~$ for h in toinoi-app-02 toinoi-bastion; do echo "== $h"; ssh $h 'uptime; df -h / | tail -n 1'; done
== toinoi-app-02
 09:40:45 up  2:42,  0 user,  load average: 0.76, 0.46, 0.40
overlay        1007G  8.9G  947G   1% /
== toinoi-bastion
 09:40:45 up  2:42,  0 user,  load average: 0.76, 0.46, 0.40
overlay        1007G  8.9G  947G   1% /
an@toinoi-app-01:~$ for h in toinoi-app-02 toinoi-bastion; do ssh $h 'grep PRETTY /etc/os-release' | sed "s/^/$h: /"; done
toinoi-app-02: PRETTY_NAME="Ubuntu 24.04.5 LTS"
toinoi-bastion: PRETTY_NAME="Ubuntu 24.04.5 LTS"
an@toinoi-app-01:~$ for h in toinoi-app-02 toinoi-khong-co; do ssh -o ConnectTimeout=3 $h hostname || echo "LOI: $h"; done
toinoi-app-02
ssh: Could not resolve hostname toinoi-khong-co: No address associated with hostname
LOI: toinoi-khong-co
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nhieu-server-ansible-toi-thieu/ssh-studio-05.webp" alt="Terminal của SSH Studio: vim ~/.ssh/config, ssh toinoi-bastion hostname thêm bastion vào known_hosts, vòng for in uptime và df -h / của toinoi-app-02 và toinoi-bastion" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · vòng lặp SSH đơn giản nhất qua hai máy</figcaption></figure>

- `ssh MÁY 'lệnh'` chạy lệnh trên máy kia rồi thoát; stdout trở về máy mình, nên nối được vào `sed`, `grep` như lệnh tại chỗ.
- Đặt lệnh trong **nháy đơn**: `$`, `|` được hiểu ở máy kia. Nháy kép thì shell của máy mình mở rộng trước.
- `| sed "s/^/$h: /"` gắn tên máy vào đầu mỗi dòng: chạy nhiều máy thì phải biết dòng nào của máy nào.
- Máy không tới được: `ssh` trả mã thoát khác 0, `||` bắt lấy và ghi lỗi, vòng lặp đi tiếp. `ConnectTimeout=3` để một máy chết không giữ cả vòng lặp hai phút.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nhieu-server-ansible-toi-thieu/ssh-studio-01.webp" alt="Terminal của SSH Studio trên toinoi-app-01: vòng for đọc PRETTY_NAME của toinoi-app-02 và toinoi-bastion đều là Ubuntu 24.04.5 LTS; vòng có toinoi-khong-co báo Could not resolve hostname và in LOI: toinoi-khong-co" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · vòng lặp SSH: một máy lỗi thì in dòng LOI, các máy khác vẫn chạy</figcaption></figure>

Vòng lặp đủ cho việc **đọc**: thu thập thông tin, kiểm tra nhanh. Việc **sửa cấu hình** trên nhiều máy thì cần thêm: chỉ sửa khi chưa đúng, kiểm tra trước khi áp, nạp lại dịch vụ khi có thay đổi, báo máy nào lỗi. Viết tất cả bằng bash rất dễ sai. Đó là việc của Ansible.

## 2. Ansible: không cài gì trên máy đích

**Ansible** chạy từ một máy điều khiển (app01), đăng nhập máy đích bằng SSH, đẩy một đoạn Python nhỏ sang, chạy và lấy kết quả về. Máy đích chỉ cần SSH và Python 3, không cần agent.

```text
an@toinoi-app-01:~$ sudo apt install -y ansible-core 2>&1 | grep -E '^Setting up ansible-core'
Setting up ansible-core (2.16.3-0ubuntu2) ...
an@toinoi-app-01:~$ ansible --version | head -n 1
ansible [core 2.16.3]
an@toinoi-app-01:~$ mkdir -p ~/van-hanh && cd ~/van-hanh
an@toinoi-app-01:~/van-hanh$ vim hosts.ini
an@toinoi-app-01:~/van-hanh$ ansible all -i hosts.ini -m ping
toinoi-app-02 | SUCCESS => {
    "changed": false,
    "ping": "pong"
}
toinoi-bastion | SUCCESS => {
    "changed": false,
    "ping": "pong"
}
an@toinoi-app-01:~/van-hanh$ ansible all -i hosts.ini -a 'uptime'
toinoi-app-02 | CHANGED | rc=0 >>
 09:41:35 up  2:43,  0 user,  load average: 0.86, 0.53, 0.43
toinoi-bastion | CHANGED | rc=0 >>
 09:41:35 up  2:43,  0 user,  load average: 0.86, 0.53, 0.43
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nhieu-server-ansible-toi-thieu/ssh-studio-06.webp" alt="Terminal của SSH Studio: apt install ansible-core 2.16.3, ansible --version, tạo ~/van-hanh, vim hosts.ini, ansible all -m ping cho toinoi-app-02 và toinoi-bastion đều SUCCESS pong" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · cài Ansible trên app01, <code>-m ping</code> kiểm tra kết nối tới mọi máy</figcaption></figure>

**Inventory** liệt kê máy và nhóm:

```ini
# Các server của Tới Nơi mà toinoi-app-01 quản lý qua SSH (LNX-056).
# Tên máy lấy theo ~/.ssh/config, nên Ansible dùng đúng user và key đồng bộ.
[app]
toinoi-app-02

[ha_tang]
toinoi-bastion

[all:vars]
ansible_python_interpreter=/usr/bin/python3
```

- `ansible all -m ping` không phải ping mạng: nó đăng nhập SSH và chạy Python, trả `pong` nếu mọi thứ sẵn sàng.
- `ansible all -a 'lệnh'` (module `command`) chạy một lệnh trên mọi máy, song song. `CHANGED` ở đây chỉ có nghĩa là lệnh đã chạy: Ansible không biết một lệnh tuỳ ý có thay đổi gì không.
- `all` có thể thay bằng tên nhóm (`app`, `ha_tang`) hay tên máy.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nhieu-server-ansible-toi-thieu/ssh-studio-02.webp" alt="Terminal của SSH Studio: ansible all -i hosts.ini -a 'uptime' trả CHANGED rc=0 cho toinoi-bastion và toinoi-app-02 kèm dòng uptime của từng máy" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · Ansible chạy một lệnh trên mọi máy trong inventory</figcaption></figure>

## 3. Playbook: cùng một chính sách cho mọi máy

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nhieu-server-ansible-toi-thieu/ssh-studio-03.webp" alt="Vim mở ~/van-hanh/ssh-toinoi.yml trong SSH Studio: play cho hosts all với become true, task copy ghi /etc/ssh/sshd_config.d/20-toinoi.conf có MaxAuthTries 3 và LoginGraceTime 30, validate bằng sshd -t, notify handler nạp lại sshd" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · playbook <code>ssh-toinoi.yml</code></figcaption></figure>

```yaml
# Cùng một chính sách SSH cho mọi server của Tới Nơi (LNX-056).
- name: Chinh sach SSH cua Toi Noi
  hosts: all
  become: true
  tasks:
    - name: Gioi han so lan thu xac thuc moi ket noi
      ansible.builtin.copy:
        dest: /etc/ssh/sshd_config.d/20-toinoi.conf
        content: |
          # Do Ansible quan ly (~/van-hanh/ssh-toinoi.yml tren toinoi-app-01). Sua o do, khong sua tay.
          MaxAuthTries 3
          LoginGraceTime 30
        owner: root
        group: root
        mode: "0644"
        validate: /usr/sbin/sshd -t -f %s
      notify: Nap lai sshd

  handlers:
    - name: Nap lai sshd
      ansible.builtin.systemd:
        name: ssh
        state: reloaded
```

| Phần | Vai trò |
|---|---|
| `become: true` | Chạy task bằng `sudo` trên máy đích |
| `copy` với `content` | Đảm bảo file có **đúng** nội dung, quyền, chủ. Đã đúng thì không làm gì |
| `validate` | Thử file mới bằng `sshd -t` **trước** khi đặt vào chỗ thật. Sai cú pháp thì dừng, file cũ còn nguyên |
| `notify` / `handlers` | Chỉ nạp lại sshd khi file thật sự thay đổi, và chỉ một lần cuối play |

`sudo` trên máy đích cần mật khẩu của `an`. Lab đặt nó trong `~/.config/ansible-become` (quyền 600) và đưa cho Ansible bằng `--become-password-file`. Trên hệ thống thật, dùng `ansible-vault` hoặc trình quản lý bí mật, và không bao giờ đưa file này vào Git.

```text
an@toinoi-app-01:~/van-hanh$ ansible-playbook -i hosts.ini ssh-toinoi.yml --become-password-file ~/.config/ansible-become

PLAY [Chinh sach SSH cua Toi Noi] ******************************************************************

TASK [Gathering Facts] *****************************************************************************
ok: [toinoi-app-02]
ok: [toinoi-bastion]

TASK [Gioi han so lan thu xac thuc moi ket noi] ****************************************************
changed: [toinoi-bastion]
changed: [toinoi-app-02]

RUNNING HANDLER [Nap lai sshd] *********************************************************************
changed: [toinoi-app-02]
changed: [toinoi-bastion]

PLAY RECAP *****************************************************************************************
toinoi-app-02              : ok=3    changed=2    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0
toinoi-bastion             : ok=3    changed=2    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0

an@toinoi-app-01:~/van-hanh$ ansible-playbook -i hosts.ini ssh-toinoi.yml --become-password-file ~/.config/ansible-become | tail -n 3
toinoi-app-02              : ok=2    changed=0    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0
toinoi-bastion             : ok=2    changed=0    unreachable=0    failed=0    skipped=0    rescued=0    ignored=0

an@toinoi-app-01:~/van-hanh$ ansible all -i hosts.ini -a 'sshd -T' --become --become-password-file ~/.config/ansible-become | grep -E 'CHANGED|SUCCESS|maxauthtries'
toinoi-app-02 | CHANGED | rc=0 >>
maxauthtries 3
toinoi-bastion | CHANGED | rc=0 >>
maxauthtries 3
```

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nhieu-server-ansible-toi-thieu/ssh-studio-07.webp" alt="Terminal của SSH Studio: vim ssh-toinoi.yml, tạo ~/.config/ansible-become quyền 600, ansible-playbook chạy lần đầu: Gathering Facts, task Gioi han so lan thu xac thuc changed, handler Nap lai sshd, PLAY RECAP changed=2 trên mỗi máy" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · playbook chạy lần đầu: đổi cấu hình và nạp lại sshd</figcaption></figure>

- Lần đầu: task `changed` trên cả hai máy, handler chạy.
- **Lần hai: `changed=0`**, handler không chạy. Đây là tính **lặp lại** (*idempotent*): chạy bao nhiêu lần cũng ra cùng trạng thái, nên playbook chạy được định kỳ để kéo máy «lệch» về đúng.
- `sshd -T` trên mọi máy xác nhận cấu hình đang có hiệu lực.

<figure class="tkh-fig"><div class="tkh-fig-box"><img src="/images/articles/linux-nhieu-server-ansible-toi-thieu/ssh-studio-04.webp" alt="Terminal của SSH Studio: chạy playbook lần hai, PLAY RECAP cho changed=0 trên cả hai máy; ansible -a 'sshd -T' --become lọc maxauthtries 3 trên toinoi-app-02 và toinoi-bastion" width="2400" height="1350" loading="lazy"></div><figcaption>SSH Studio · chạy lại không đổi gì, cả hai máy đã có <code>maxauthtries 3</code></figcaption></figure>

## 4. Lỗi thường gặp

**Vòng lặp `ssh` sửa cấu hình bằng `sed -i`.** Chạy hai lần là sửa hai lần, sai một máy không ai biết. Dùng playbook.

**Dùng module `command`/`shell` cho mọi việc.** Mất tính lặp lại; Ansible có module cho file, gói, dịch vụ, người dùng.

**Sửa tay file do Ansible quản lý.** Lần chạy sau ghi đè. Dòng chú thích đầu file nói rõ nó do đâu quản lý.

**Thiếu `validate` cho cấu hình sshd.** File sai đẩy ra mọi máy cùng lúc, lần nạp lại sau khoá mọi người ngoài.

## Bài tập

1. `ansible all -i hosts.ini -m setup -a 'filter=ansible_distribution*'`. `setup` thu được những gì?
2. Thêm vào playbook task cài `htop` bằng `ansible.builtin.apt` (`update_cache: true`). Chạy hai lần.
3. Chạy playbook với `--check --diff` sau khi sửa `MaxAuthTries` thành 4. Ansible cho thấy gì mà chưa đổi gì?
4. Thêm app01 vào inventory với `ansible_connection=local`. Vì sao cần dòng này, thay vì để Ansible đăng nhập SSH vào chính app01 (gợi ý: sshd của app01 đòi key **và** OTP)?

## Đọc tiếp

[LNX-057 · Triển khai bằng thư mục release và symlink](/articles/linux-trien-khai-release-symlink).

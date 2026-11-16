-- P01 · 00-reset: xoá keyspace toinoi cũ (nếu có) rồi tạo lại từ đầu.
-- Chỉ chạy trên lab. DROP KEYSPACE xoá toàn bộ bảng và dữ liệu của keyspace, không hỏi lại
-- (Cassandra mặc định còn chụp snapshot tự động trước khi xoá: auto_snapshot = true — CS-045).

-- ## Phiên bản và vị trí của phiên
SELECT cluster_name, release_version, data_center, rack, cql_version FROM system.local;

-- ## Xoá keyspace cũ và tạo lại
DROP KEYSPACE IF EXISTS toinoi;

-- Một datacenter tên dc1, mỗi dòng có 1 bản sao vì lab P01 chỉ có một node.
-- Phần 3 dựng cụm 3 node và nâng lên 3 bản sao (CS-023).
CREATE KEYSPACE toinoi
  WITH replication = {'class': 'NetworkTopologyStrategy', 'dc1': 1}
  AND durable_writes = true;

SELECT keyspace_name, durable_writes, replication FROM system_schema.keyspaces WHERE keyspace_name = 'toinoi';

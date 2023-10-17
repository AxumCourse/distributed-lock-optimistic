CREATE TABLE inventory ( -- 库存
	id SERIAL PRIMARY KEY,
	stock INTEGER NOT NULL, -- 库存数量
	ver INTEGER NOT NULL DEFAULT 0 -- 乐观锁的版本号
);
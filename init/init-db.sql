-- Create the test database if it doesn't exist
SELECT 'CREATE DATABASE koko_pic_test' WHERE NOT EXISTS (
    SELECT FROM pg_database WHERE datname = 'koko_pic_test'
)\gexec
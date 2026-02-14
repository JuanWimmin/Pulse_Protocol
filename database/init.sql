-- Extensiones necesarias para UUIDs y crypto
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Configuración de timezone
SET timezone = 'UTC';

-- Log para confirmar que se ejecutó
DO $$ 
BEGIN
    RAISE NOTICE 'Database initialized successfully';
END $$;
CREATE TYPE item_category AS ENUM ('gui_app', 'cli_tool', 'framework', 'language');
CREATE TYPE item_source AS ENUM ('brew', 'cask', 'npm', 'cargo', 'app');

CREATE TABLE users (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    username text UNIQUE NOT NULL,
    email text UNIQUE NOT NULL,
    password_hash text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE api_tokens (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name text NOT NULL,
    token_hash text UNIQUE NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);
CREATE INDEX api_tokens_user_id_idx ON api_tokens (user_id);

CREATE TABLE tech_stacks (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    custom_nickname text NOT NULL,
    slug text UNIQUE NOT NULL,
    os_version text,
    chip text,
    cpu_model text,
    ram_gb int,
    hardware_info jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX tech_stacks_user_id_idx ON tech_stacks (user_id);

CREATE TABLE software_items (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tech_stack_id uuid NOT NULL REFERENCES tech_stacks (id) ON DELETE CASCADE,
    name text NOT NULL,
    category item_category NOT NULL,
    source item_source NOT NULL,
    version text,
    install_path text
);
CREATE INDEX software_items_tech_stack_id_idx ON software_items (tech_stack_id);

CREATE TABLE stack_battles (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    challenger_stack_id uuid NOT NULL REFERENCES tech_stacks (id) ON DELETE CASCADE,
    opponent_stack_id uuid NOT NULL REFERENCES tech_stacks (id) ON DELETE CASCADE,
    winner_stack_id uuid REFERENCES tech_stacks (id) ON DELETE SET NULL,
    score jsonb NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);

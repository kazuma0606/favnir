"""v34.6A: !Effect → ctx 移行スクリプト（改訂版）
runes/ 配下の .fav ファイルの !Effect アノテーションを Capability Context に移行する。
対応パターン:
  1. 1 行シグネチャ:  fn f(args) -> R !Eff {
  2. 多行シグネチャ:  ) -> R !Eff {  （パラメータが複数行にわたる場合の最終行）
  3. = 構文:          fn f(args) -> R !Eff =
"""
import re
import os
import sys

EFFECT_TO_FIELD = {
    'Postgres': 'db',
    'Redis': 'redis',
    'Http': 'http',
    'Network': 'http',
    'Rpc': 'http',
    'Gcp': 'http',
    'Io': 'io',
    'File': 'io',
    'AzureStorage': 'io',
    'Stream': 'stream',
    'Checkpoint': 'stream',
    'PipelineState': 'stream',
    'Trace': 'tracer',
    'Snowflake': 'warehouse',
    'Llm': 'llm',
    'Db': 'db',
    'DbRead': 'db',
    'DbWrite': 'db',
    'DbAdmin': 'db',
    'MySQL': 'db',
    'MongoDB': 'db',
    'DynamoDB': 'db',
    'Elasticsearch': 'db',
    'AzureDb': 'db',
}

def get_fields_for_effects(effects_str):
    effects = re.findall(r'!(\w+)', effects_str)
    fields = []
    for eff in effects:
        field = EFFECT_TO_FIELD.get(eff)
        if field and field not in fields:
            fields.append(field)
    return fields

def make_bind(fields):
    if not fields:
        return None
    return "    bind { " + ", ".join(fields) + " } <- ctx"

def migrate_content(content):
    lines = content.split('\n')
    result = []
    changed = False

    # パターン 1: fn f(params) -> R !Eff1 !Eff2 {
    # パターン 2: ) -> R !Eff1 {   (多行シグネチャの最終行)
    # パターン 3: fn f(params) -> R !Eff =  (= 構文)
    SINGLE_LINE_BRACE  = re.compile(
        r'^(\s*(?:public\s+)?fn\s+\S+\s*\()([^)]*)\)\s*(->[^{!]*)(\s*(?:!\w+\s*)+)\{'
    )
    MULTILINE_CLOSE = re.compile(
        r'^(\s*\)\s*)(->[^{!]*)(\s*(?:!\w+\s*)+)\{'
    )
    SINGLE_LINE_EQ = re.compile(
        r'^(\s*(?:public\s+)?fn\s+\S+\s*\()([^)]*)\)\s*(->[^=!]*)(\s*(?:!\w+\s*)+)=\s*$'
    )
    MULTILINE_CLOSE_EQ = re.compile(
        r'^(\s*\)\s*)(->[^=!]*)(\s*(?:!\w+\s*)+)=\s*$'
    )

    i = 0
    while i < len(lines):
        line = lines[i]

        # --- パターン 1: 1 行 { ---
        m = SINGLE_LINE_BRACE.match(line)
        if m:
            prefix   = m.group(1)
            params   = m.group(2).strip()
            ret_part = m.group(3).rstrip()
            eff_str  = m.group(4)
            fields   = get_fields_for_effects(eff_str)
            new_params = ("ctx: AppCtx, " + params) if params else "ctx: AppCtx"
            result.append(f"{prefix}{new_params}){ret_part} {{")
            bind = make_bind(fields)
            if bind:
                result.append(bind)
            changed = True
            i += 1
            continue

        # --- パターン 2: 多行閉じ { ---
        m = MULTILINE_CLOSE.match(line)
        if m:
            close    = m.group(1)   # "    ) "
            ret_part = m.group(2).rstrip()
            eff_str  = m.group(3)
            fields   = get_fields_for_effects(eff_str)
            # 先行行（多行シグネチャの最初の fn 行）に ctx: AppCtx を追加する必要がある
            # → result の最後の fn f( 行を探して ctx: AppCtx を先頭パラメータとして追加
            fn_idx = len(result) - 1
            while fn_idx >= 0:
                prev = result[fn_idx]
                fn_m = re.match(r'^(\s*(?:public\s+)?fn\s+\S+\s*\()(.*)', prev)
                if fn_m:
                    fn_prefix = fn_m.group(1)
                    fn_rest   = fn_m.group(2)
                    # 既に ctx: AppCtx が含まれていなければ追加
                    if 'ctx: AppCtx' not in prev:
                        result[fn_idx] = fn_prefix + "ctx: AppCtx,\n" + fn_rest
                    break
                fn_idx -= 1
            result.append(f"{close}{ret_part} {{")
            bind = make_bind(fields)
            if bind:
                result.append(bind)
            changed = True
            i += 1
            continue

        # --- パターン 3: 1 行 = ---
        m = SINGLE_LINE_EQ.match(line)
        if m:
            prefix   = m.group(1)
            params   = m.group(2).strip()
            ret_part = m.group(3).rstrip()
            eff_str  = m.group(4)
            fields   = get_fields_for_effects(eff_str)
            new_params = ("ctx: AppCtx, " + params) if params else "ctx: AppCtx"
            # = 構文を { } に変換 (次の行を本体として扱う)
            result.append(f"{prefix}{new_params}){ret_part} {{")
            bind = make_bind(fields)
            if bind:
                result.append(bind)
            # 次の行（本体）をそのまま取り込み、末尾に } を追加
            i += 1
            # 本体が複数行にわたる可能性がある場合は末尾の空行まで取り込む
            body_lines = []
            while i < len(lines):
                bline = lines[i]
                body_lines.append(bline)
                i += 1
                # 空行または次の fn 定義が来たら終了
                if bline.strip() == '' or re.match(r'^\s*(?:public\s+)?fn\s+', bline):
                    break
            # 最後に } を追加
            for bl in body_lines[:-1]:
                result.append(bl)
            result.append("}")
            if body_lines:
                result.append(body_lines[-1])
            changed = True
            continue

        # --- パターン 4: 多行 = ---
        m = MULTILINE_CLOSE_EQ.match(line)
        if m:
            close    = m.group(1)
            ret_part = m.group(2).rstrip()
            eff_str  = m.group(3)
            fields   = get_fields_for_effects(eff_str)
            fn_idx = len(result) - 1
            while fn_idx >= 0:
                prev = result[fn_idx]
                fn_m = re.match(r'^(\s*(?:public\s+)?fn\s+\S+\s*\()(.*)', prev)
                if fn_m:
                    fn_prefix = fn_m.group(1)
                    fn_rest   = fn_m.group(2)
                    if 'ctx: AppCtx' not in prev:
                        result[fn_idx] = fn_prefix + "ctx: AppCtx,\n" + fn_rest
                    break
                fn_idx -= 1
            result.append(f"{close}{ret_part} {{")
            bind = make_bind(fields)
            if bind:
                result.append(bind)
            i += 1
            body_lines = []
            while i < len(lines):
                bline = lines[i]
                body_lines.append(bline)
                i += 1
                if bline.strip() == '' or re.match(r'^\s*(?:public\s+)?fn\s+', bline):
                    break
            for bl in body_lines[:-1]:
                result.append(bl)
            result.append("}")
            if body_lines:
                result.append(body_lines[-1])
            changed = True
            continue

        result.append(line)
        i += 1

    return '\n'.join(result), changed


def has_effect_annotation(content):
    """1 行内で -> ... !Word ... { または = を持つかチェック"""
    for line in content.split('\n'):
        if re.search(r'->[^#\n]*!\w+[^#\n]*[{=]', line):
            return True
    return False


def process_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    if not has_effect_annotation(content):
        return False

    new_content, changed = migrate_content(content)
    if changed:
        with open(filepath, 'w', encoding='utf-8', newline='') as f:
            f.write(new_content)
        return True
    return False


def find_rune_files(runes_dir):
    result = []
    ctx_dir = os.path.join(runes_dir, 'ctx')
    for root, dirs, files in os.walk(runes_dir):
        dirs[:] = [d for d in dirs if os.path.normpath(os.path.join(root, d)) != os.path.normpath(ctx_dir)]
        for fname in files:
            if fname.endswith('.fav'):
                result.append(os.path.join(root, fname))
    return sorted(result)


def main():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.dirname(script_dir)
    runes_dir = os.path.join(project_root, 'runes')

    files = find_rune_files(runes_dir)
    print(f"Found {len(files)} .fav files in runes/ (excluding ctx/)")

    migrated = []
    for fp in files:
        if process_file(fp):
            rel = os.path.relpath(fp, project_root)
            migrated.append(rel)
            print(f"  migrated: {rel}")

    print(f"\nMigrated: {len(migrated)} files")

    remaining = []
    for fp in files:
        with open(fp, 'r', encoding='utf-8') as f:
            content = f.read()
        if has_effect_annotation(content):
            remaining.append(os.path.relpath(fp, project_root))

    if remaining:
        print(f"\nWARNING: {len(remaining)} files still have !Effect:")
        for r in remaining:
            print(f"  {r}")
        # 各ファイルの残存行を表示
        for fp in files:
            with open(fp, 'r', encoding='utf-8') as f:
                content = f.read()
            rel = os.path.relpath(fp, project_root)
            if rel in remaining:
                for lineno, line in enumerate(content.split('\n'), 1):
                    if re.search(r'->[^#\n]*!\w+[^#\n]*[{=]', line):
                        print(f"    {rel}:{lineno}: {line.strip()[:80]}")
        sys.exit(1)
    else:
        print("\nOK: No !Effect annotations remain in runes/")


if __name__ == '__main__':
    main()

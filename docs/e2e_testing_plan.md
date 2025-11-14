# DataWise Desktop 端到端测试计划
版本：v1.0  
日期：2025-11-14  
适用范围：Tauri、egui、tui 三个 UI 框架

---

## 1. 测试范围

### 1.1 Tauri 端到端测试

**工具**：Playwright + tauri-driver

**测试场景**：
- [ ] 应用启动和初始化
- [ ] SQL 编辑器输入和执行
- [ ] 文件导入（CSV/Parquet）
- [ ] 结果展示和导出
- [ ] 错误处理和提示
- [ ] 快捷键操作

### 1.2 egui 端到端测试

**工具**：自定义测试框架 + 截图对比

**测试场景**：
- [ ] 窗口创建和布局
- [ ] SQL 输入和执行
- [ ] 结果表格渲染
- [ ] 文件拖放导入
- [ ] 错误提示显示

### 1.3 tui 端到端测试

**工具**：自定义测试框架 + 终端模拟

**测试场景**：
- [ ] 终端初始化
- [ ] 键盘输入处理
- [ ] SQL 执行和结果显示
- [ ] 快捷键操作
- [ ] 错误提示显示

---

## 2. Tauri 端到端测试实现

### 2.1 环境设置

```bash
# 安装依赖
npm install --save-dev @playwright/test @tauri-apps/cli

# 配置 playwright.config.ts
```

### 2.2 测试用例

```typescript
// tests/e2e/sql-execution.spec.ts
import { test, expect } from '@playwright/test';

test('Execute simple SQL query', async ({ page }) => {
  // 1. 启动应用
  // 2. 输入 SQL
  // 3. 点击执行
  // 4. 验证结果
});

test('Import CSV file', async ({ page }) => {
  // 1. 拖放 CSV 文件
  // 2. 验证导入成功提示
  // 3. 验证文件树更新
});

test('Export query result', async ({ page }) => {
  // 1. 执行查询
  // 2. 点击导出
  // 3. 选择格式和路径
  // 4. 验证文件生成
});
```

### 2.3 运行测试

```bash
# 运行所有 E2E 测试
npm run test:e2e

# 运行特定测试
npm run test:e2e -- sql-execution.spec.ts

# 调试模式
npm run test:e2e -- --debug
```

---

## 3. Core 单元测试补充

### 3.1 新增测试用例

- [ ] 测试 Cancel 命令的正确性
- [ ] 测试并发任务处理
- [ ] 测试大文件导入的进度事件
- [ ] 测试导出格式转换
- [ ] 测试 SQL 错误处理

### 3.2 测试覆盖率目标

- **目标**：> 80%
- **当前**：~70%
- **计划**：添加 10+ 新测试用例

---

## 4. 集成测试补充

### 4.1 跨 UI 集成测试

- [ ] 验证所有 UI 都能正确调用 Core API
- [ ] 验证事件流正确传递
- [ ] 验证错误处理一致性

### 4.2 性能集成测试

- [ ] 验证 1GB CSV 导入性能
- [ ] 验证 100 万行渲染性能
- [ ] 验证查询响应时间

---

## 5. CI/CD 集成

### 5.1 GitHub Actions 配置

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run E2E tests
        run: npm run test:e2e
```

### 5.2 测试报告

- 生成 HTML 测试报告
- 上传到 GitHub Pages
- 发送测试结果通知

---

## 6. 测试数据管理

### 6.1 测试数据生成

```rust
// datawise-core/src/test_data.rs
pub fn generate_test_csv(rows: usize) -> String {
    // 生成测试 CSV 数据
}

pub fn generate_test_parquet(rows: usize) -> Vec<u8> {
    // 生成测试 Parquet 数据
}
```

### 6.2 测试数据存储

```
tests/fixtures/
├── sample.csv
├── sample.parquet
├── large_1gb.csv
└── README.md
```

---

## 7. 测试执行计划

### 第 1 周
- [ ] 设置 Playwright 环境
- [ ] 编写基础 E2E 测试
- [ ] 配置 CI/CD

### 第 2 周
- [ ] 补充 Core 单元测试
- [ ] 编写集成测试
- [ ] 性能测试

### 第 3 周
- [ ] 跨平台测试
- [ ] 测试报告生成
- [ ] 文档完善

---

## 8. 成功标准

- ✅ 所有 E2E 测试通过
- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试通过率 100%
- ✅ 跨平台兼容性验证
- ✅ 性能指标达到目标



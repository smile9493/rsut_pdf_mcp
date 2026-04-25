获取缓存统计信息，用于性能监控和调优。

## 返回格式

```json
{
  "size": 150,
  "max_size": 1000,
  "hits": 1250,
  "misses": 150,
  "hit_rate": 0.89
}
```

## 性能建议

- 命中率 < 70%：考虑增加缓存大小（CACHE_MAX_SIZE）
- 命中率 > 95%：缓存大小充足
- 缓存满：考虑增加 TTL（CACHE_TTL_SECONDS）

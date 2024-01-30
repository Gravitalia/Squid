# Squid 🦑
> Like the number of squid species and their **agility**, Squid adapts to all systems to analyze in **real time** the **messages** & **hashtags**.

## Goals
- Trend detection
- Ranking of recent hashtags
- Detection of propaganda content (*soon*)

Squid **is not** a search engine as such.

## Start Squid
### `docker-compose.yaml`
```yaml
squid:
  image: ghcr.io/gravitalia/squid
  container_name: Squid
  ports:
    - 5555:5555
  volumes:
    - ./config.yaml:/config.yaml
```

### From source
1. Clone this repository with `git`:
   
   ```
   git clone https://github.com/Gravitalia/Squid
   cd squid
   ```
3. Build with `bazel`:
   ```
   bazel build //...
   ```
4. Run with `bazel`:
   ```
   bazel run //squid
   ```

## License
[Apache 2.0](https://github.com/Gravitalia/Squid/blob/master/LICENSE)

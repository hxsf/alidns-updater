# alidns-updater

A service to update alidns

> - [aliyun API](https://next.api.alibabacloud.com/api/Alidns/2015-01-09/DescribeSubDomainRecords?tab=DEBUG&lang=TYPESCRIPT)
> - [records](https://dns.console.aliyun.com/)

## Usage

### Setup

[docker-compose](./docker-compose.yaml) 

### API

- Get all DNS records: `GET /dns`
    ``` shell
    curl http://127.0.0.1:8080/dns
    ```
- Get single DNS record: `GET /dns/:rr`
    ``` shell
    # get the aaa.<your-domain>
    curl http://127.0.0.1:8080/dns/aaa
    ```
- Add or Replace single recored: `POST /dns/:rr` with `ip=xxx` in body or query.
    ``` shell
    # set the aaa.<your-domain> = 1.2.3.4
    curl -X POST http://127.0.0.1:8080/dns/aaa?ip=1.2.3.4
    curl -X POST http://127.0.0.1:8080/dns/aaa -d '{"ip": "1.2.3.4"}'
    ```

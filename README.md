### 🔧 Environment Variables

`.env` 파일을 프로젝트 루트에 생성하고 아래 내용을 추가해주세요:

```
# Database 연결 URL (예: MySQL)
DATABASE_URL=mysql://<user>:<password>@<host>:<port>/<database>

# JWT 서명에 사용할 시크릿 키
JWT_SECRET=your_jwt_secret
```
import { useState } from "react";
import { useNavigate, useSearchParams } from "react-router";
import { Button, Card, CardBody, CardHeader, Input, Alert } from "@heroui/react";
import { login, type LoginRequest } from "../api";
import { useAuth } from "../auth-context";

export default function Login() {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const { login: authLogin } = useAuth();

  const redirectTo = searchParams.get("redirect") || "/";

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");

    setLoading(true);

    try {
      const response = await login({
        username,
        password,
      } as LoginRequest);

      // 使用 auth context 保存登录状态
      authLogin(response.token, response.member);

      // 跳转到目标页面
      navigate(redirectTo);
    } catch (err) {
      setError("Invalid username or password");
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="min-h-screen flex items-center justify-center">
      <Card className="w-full max-w-md">
        <CardHeader className="flex flex-col gap-2 pb-0">
          <h1 className="text-2xl font-bold">Login to HomeDrive</h1>
          <p className="text-default-500">
            Enter your credentials to access your files
          </p>
        </CardHeader>
        <CardBody>
          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            {error && (
              <Alert color="danger">
                <div className="ml-2">{error}</div>
              </Alert>
            )}

            <Input
              label="Username"
              placeholder="Enter your username"
              value={username}
              onValueChange={setUsername}
              required
            />

            <Input
              label="Password"
              placeholder="Enter your password"
              type="password"
              value={password}
              onValueChange={setPassword}
              required
            />

            <Button
              type="submit"
              color="primary"
              className="w-full"
              isLoading={loading}
            >
              Login
            </Button>
          </form>
        </CardBody>
      </Card>
    </main>
  );
}

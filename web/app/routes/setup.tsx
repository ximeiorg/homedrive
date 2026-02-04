import { useState } from "react";
import { useNavigate } from "react-router";
import { Button, Card, CardBody, CardHeader, Input, Alert } from "@heroui/react";
import { initAdmin, type InitAdminRequest } from "../api";

export function meta() {
  return [
    { title: "Setup - HomeDrive" },
    { name: "description", content: "Set up your administrator account" },
  ];
}

interface FormData {
  username: string;
  password: string;
  confirmPassword: string;
  storageTag: string;
}

interface FormErrors {
  username?: string;
  password?: string;
  storageTag?: string;
  general?: string;
}

export default function Setup() {
  const navigate = useNavigate();
  const [formData, setFormData] = useState<FormData>({
    username: "",
    password: "",
    confirmPassword: "",
    storageTag: "",
  });
  const [errors, setErrors] = useState<FormErrors>({});
  const [loading, setLoading] = useState(false);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
    // 清除对应字段的错误
    if (errors[name as keyof FormErrors]) {
      setErrors((prev) => ({ ...prev, [name]: undefined }));
    }
  };

  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    if (!formData.username.trim()) {
      newErrors.username = "Username is required";
    }

    if (formData.password.length < 6) {
      newErrors.password = "Password must be at least 6 characters";
    }

    if (formData.password !== formData.confirmPassword) {
      newErrors.password = "Passwords do not match";
    }

    if (!formData.storageTag.trim()) {
      newErrors.storageTag = "Storage tag is required";
    } else if (formData.storageTag.length > 50) {
      newErrors.storageTag = "Storage tag must be less than 50 characters";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validateForm()) {
      return;
    }

    setLoading(true);
    setErrors({});

    try {
      console.log("Submitting form data:", {
        username: formData.username,
        storage_tag: formData.storageTag,
      });

      const response = await initAdmin({
        username: formData.username,
        password: formData.password,
        storage_tag: formData.storageTag,
      } as InitAdminRequest);

      console.log("API Response:", response);

      if (response.success) {
        console.log("Admin user created successfully, redirecting to /login");
        navigate("/login");
      } else {
        console.error("API returned error:", response.message);
        setErrors({ general: response.message || "Failed to create admin user" });
      }
    } catch (err) {
      console.error("Failed to create admin user:", err);
      console.error("Error details:", {
        message: err instanceof Error ? err.message : String(err),
        stack: err instanceof Error ? err.stack : undefined,
      });
      setErrors({
        general: "Failed to create admin user: " + (err instanceof Error ? err.message : String(err)),
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <main className="min-h-screen flex items-center justify-center">
      <Card className="w-full max-w-md">
        <CardHeader className="flex flex-col gap-2 pb-0">
          <h1 className="text-2xl font-bold">Welcome to HomeDrive</h1>
          <p className="text-default-500">
            Set up your administrator account to get started
          </p>
        </CardHeader>
        <CardBody>
          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            {errors.general && (
              <Alert color="danger">
                <div className="ml-2">{errors.general}</div>
              </Alert>
            )}

            <Input
              label="Username"
              name="username"
              placeholder="Enter your username"
              value={formData.username}
              onChange={handleChange}
              isInvalid={!!errors.username}
              errorMessage={errors.username}
              required
            />

            <Input
              label="Storage Tag"
              name="storageTag"
              placeholder="Enter your storage directory name"
              description="This will be used as your storage directory name"
              value={formData.storageTag}
              onChange={handleChange}
              isInvalid={!!errors.storageTag}
              errorMessage={errors.storageTag}
              required
            />

            <Input
              label="Password"
              name="password"
              placeholder="Enter your password"
              type="password"
              value={formData.password}
              onChange={handleChange}
              isInvalid={!!errors.password}
              errorMessage={errors.password}
              required
            />

            <Input
              label="Confirm Password"
              name="confirmPassword"
              placeholder="Confirm your password"
              type="password"
              value={formData.confirmPassword}
              onChange={handleChange}
              isInvalid={!!errors.password && formData.password !== formData.confirmPassword}
              errorMessage={errors.password && formData.password !== formData.confirmPassword ? "Passwords do not match" : undefined}
              required
            />

            <Button
              type="submit"
              color="primary"
              className="w-full"
              isLoading={loading}
            >
              Create Admin Account
            </Button>
          </form>
        </CardBody>
      </Card>
    </main>
  );
}

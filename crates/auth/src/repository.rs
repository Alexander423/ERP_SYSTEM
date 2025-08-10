use crate::models::{Permission, Role, Tenant, User, UserRole};
use chrono::{DateTime, Utc};
use erp_core::{DatabasePool, Error, Result, TenantContext};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthRepository {
    db: DatabasePool,
}

impl AuthRepository {
    pub fn new(db: DatabasePool) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &DatabasePool {
        &self.db
    }

    pub async fn create_tenant(&self, name: &str, schema_name: &str) -> Result<Tenant> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "INSERT INTO public.tenants (name, schema_name) VALUES ($1, $2) RETURNING *"
        )
        .bind(name)
        .bind(schema_name)
        .fetch_one(&self.db.main_pool)
        .await?;

        self.db.create_tenant_schema(schema_name).await?;

        Ok(tenant)
    }

    pub async fn get_tenant_by_id(&self, id: Uuid) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM public.tenants WHERE id = $1 AND status = 'active'"
        )
        .bind(id)
        .fetch_optional(&self.db.main_pool)
        .await?;

        Ok(tenant)
    }

    pub async fn get_tenant_by_schema(&self, schema_name: &str) -> Result<Option<Tenant>> {
        let tenant = sqlx::query_as::<_, Tenant>(
            "SELECT * FROM public.tenants WHERE schema_name = $1 AND status = 'active'"
        )
        .bind(schema_name)
        .fetch_optional(&self.db.main_pool)
        .await?;

        Ok(tenant)
    }

    pub async fn create_user(
        &self,
        tenant: &TenantContext,
        email: &str,
        password_hash: Option<&str>,
        first_name: &str,
        last_name: &str,
    ) -> Result<User> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (email, password_hash, first_name, last_name) 
             VALUES ($1, $2, $3, $4) RETURNING *"
        )
        .bind(email)
        .bind(password_hash)
        .bind(first_name)
        .bind(last_name)
        .fetch_one(pool.get())
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(
        &self,
        tenant: &TenantContext,
        email: &str,
    ) -> Result<Option<User>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(pool.get())
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<Option<User>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(user_id)
        .fetch_optional(pool.get())
        .await?;

        Ok(user)
    }

    pub async fn update_user_login(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        sqlx::query(
            "UPDATE users SET last_login_at = $1 WHERE id = $2"
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool.get())
        .await?;

        Ok(())
    }

    pub async fn lock_user(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        until: DateTime<Utc>,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        sqlx::query(
            "UPDATE users SET locked_until = $1 WHERE id = $2"
        )
        .bind(until)
        .bind(user_id)
        .execute(pool.get())
        .await?;

        Ok(())
    }

    pub async fn get_user_roles(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<Vec<Role>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let roles = sqlx::query_as::<_, Role>(
            "SELECT r.* FROM roles r 
             INNER JOIN user_roles ur ON r.id = ur.role_id 
             WHERE ur.user_id = $1"
        )
        .bind(user_id)
        .fetch_all(pool.get())
        .await?;

        Ok(roles)
    }

    pub async fn get_user_permissions(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<Vec<Permission>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT DISTINCT p.* FROM permissions p 
             INNER JOIN role_permissions rp ON p.id = rp.permission_id 
             INNER JOIN user_roles ur ON rp.role_id = ur.role_id 
             WHERE ur.user_id = $1"
        )
        .bind(user_id)
        .fetch_all(pool.get())
        .await?;

        Ok(permissions)
    }

    pub async fn assign_role_to_user(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) 
             ON CONFLICT DO NOTHING"
        )
        .bind(user_id)
        .bind(role_id)
        .execute(pool.get())
        .await?;

        Ok(())
    }

    pub async fn remove_role_from_user(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let result = sqlx::query(
            "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2"
        )
        .bind(user_id)
        .bind(role_id)
        .execute(pool.get())
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found("User role assignment not found"));
        }

        Ok(())
    }

    pub async fn get_role_by_name(
        &self,
        tenant: &TenantContext,
        name: &str,
    ) -> Result<Option<Role>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let role = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(pool.get())
        .await?;

        Ok(role)
    }

    pub async fn save_2fa_secret(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        encrypted_secret: &str,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        sqlx::query(
            "UPDATE users SET two_factor_secret_encrypted = $1, 
             two_factor_enabled_at = $2 WHERE id = $3"
        )
        .bind(encrypted_secret)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool.get())
        .await?;

        Ok(())
    }

    pub async fn remove_2fa(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        sqlx::query(
            "UPDATE users SET two_factor_secret_encrypted = NULL, 
             two_factor_enabled_at = NULL WHERE id = $1"
        )
        .bind(user_id)
        .execute(pool.get())
        .await?;

        Ok(())
    }

    pub async fn list_users(
        &self,
        tenant: &TenantContext,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<User>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.get())
        .await?;

        Ok(users)
    }

    pub async fn count_users(&self, tenant: &TenantContext) -> Result<i64> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users"
        )
        .fetch_one(pool.get())
        .await?;

        Ok(count)
    }

    pub async fn get_all_permissions(&self, tenant: &TenantContext) -> Result<Vec<Permission>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT * FROM permissions ORDER BY resource, action"
        )
        .fetch_all(pool.get())
        .await?;

        Ok(permissions)
    }

    // Additional methods for workflows

    pub async fn find_by_email(
        &self,
        tenant: &TenantContext,
        email: &str,
    ) -> Result<Option<User>> {
        self.get_user_by_email(tenant, email).await
    }

    pub async fn find_by_id(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<Option<User>> {
        self.get_user_by_id(tenant, user_id).await
    }

    pub async fn update_password(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        sqlx::query(
            "UPDATE users SET password_hash = $1, password_changed_at = $2 WHERE id = $3"
        )
        .bind(password_hash)
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool.get())
        .await?;

        Ok(())
    }

    pub async fn mark_email_verified(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<User> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET email_verified_at = $1 WHERE id = $2 RETURNING *"
        )
        .bind(Utc::now())
        .bind(user_id)
        .fetch_one(pool.get())
        .await?;

        Ok(user)
    }

    // User Management Repository Methods

    /// Updates user information.
    pub async fn update_user(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
        request: &crate::dto::UpdateUserRequest,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let mut update_parts = Vec::new();
        let mut bind_index = 1;
        
        let mut query = "UPDATE users SET updated_at = CURRENT_TIMESTAMP".to_string();
        
        if let Some(first_name) = &request.first_name {
            update_parts.push(format!(", first_name = ${}", bind_index));
            bind_index += 1;
        }
        
        if let Some(last_name) = &request.last_name {
            update_parts.push(format!(", last_name = ${}", bind_index));
            bind_index += 1;
        }
        
        if let Some(is_active) = request.is_active {
            update_parts.push(format!(", is_active = ${}", bind_index));
            bind_index += 1;
        }
        
        query.push_str(&update_parts.join(""));
        query.push_str(&format!(" WHERE id = ${}", bind_index));
        
        let mut sqlx_query = sqlx::query(&query);
        
        if let Some(first_name) = &request.first_name {
            sqlx_query = sqlx_query.bind(first_name);
        }
        
        if let Some(last_name) = &request.last_name {
            sqlx_query = sqlx_query.bind(last_name);
        }
        
        if let Some(is_active) = request.is_active {
            sqlx_query = sqlx_query.bind(is_active);
        }
        
        sqlx_query = sqlx_query.bind(user_id);
        
        let result = sqlx_query.execute(pool.get()).await?;
        
        if result.rows_affected() == 0 {
            return Err(Error::not_found("User not found"));
        }
        
        Ok(())
    }

    /// Soft deletes a user by setting deleted_at timestamp.
    pub async fn soft_delete_user(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let result = sqlx::query(
            "UPDATE users SET deleted_at = CURRENT_TIMESTAMP, is_active = false WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(user_id)
        .execute(pool.get())
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(Error::not_found("User not found or already deleted"));
        }
        
        Ok(())
    }

    /// Gets all users assigned to a specific role.
    pub async fn get_users_with_role(
        &self,
        tenant: &TenantContext,
        role_id: Uuid,
    ) -> Result<Vec<User>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let users = sqlx::query_as::<_, User>(
            "SELECT u.* FROM users u 
             INNER JOIN user_roles ur ON u.id = ur.user_id 
             WHERE ur.role_id = $1 AND u.deleted_at IS NULL
             ORDER BY u.created_at"
        )
        .bind(role_id)
        .fetch_all(pool.get())
        .await?;
        
        Ok(users)
    }

    // Role Management Repository Methods

    /// Lists all roles in the tenant.
    pub async fn list_roles(&self, tenant: &TenantContext) -> Result<Vec<Role>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let roles = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles ORDER BY name"
        )
        .fetch_all(pool.get())
        .await?;
        
        Ok(roles)
    }

    /// Gets a role by ID.
    pub async fn get_role_by_id(
        &self,
        tenant: &TenantContext,
        role_id: Uuid,
    ) -> Result<Option<Role>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let role = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE id = $1"
        )
        .bind(role_id)
        .fetch_optional(pool.get())
        .await?;
        
        Ok(role)
    }

    /// Creates a new role.
    pub async fn create_role(
        &self,
        tenant: &TenantContext,
        name: &str,
        description: Option<&str>,
        is_editable: bool,
    ) -> Result<Role> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let role = sqlx::query_as::<_, Role>(
            "INSERT INTO roles (name, description, is_editable) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(name)
        .bind(description)
        .bind(is_editable)
        .fetch_one(pool.get())
        .await?;
        
        Ok(role)
    }

    /// Updates a role.
    pub async fn update_role(
        &self,
        tenant: &TenantContext,
        role_id: Uuid,
        request: &crate::dto::UpdateRoleRequest,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let mut update_parts = Vec::new();
        let mut bind_index = 1;
        
        let mut query = "UPDATE roles SET updated_at = CURRENT_TIMESTAMP".to_string();
        
        if let Some(name) = &request.name {
            update_parts.push(format!(", name = ${}", bind_index));
            bind_index += 1;
        }
        
        if let Some(description) = &request.description {
            update_parts.push(format!(", description = ${}", bind_index));
            bind_index += 1;
        }
        
        query.push_str(&update_parts.join(""));
        query.push_str(&format!(" WHERE id = ${}", bind_index));
        
        let mut sqlx_query = sqlx::query(&query);
        
        if let Some(name) = &request.name {
            sqlx_query = sqlx_query.bind(name);
        }
        
        if let Some(description) = &request.description {
            sqlx_query = sqlx_query.bind(description);
        }
        
        sqlx_query = sqlx_query.bind(role_id);
        
        let result = sqlx_query.execute(pool.get()).await?;
        
        if result.rows_affected() == 0 {
            return Err(Error::not_found("Role not found"));
        }
        
        Ok(())
    }

    /// Deletes a role.
    pub async fn delete_role(&self, tenant: &TenantContext, role_id: Uuid) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let result = sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(role_id)
            .execute(pool.get())
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(Error::not_found("Role not found"));
        }
        
        Ok(())
    }

    /// Removes all permissions from a role.
    pub async fn remove_all_permissions_from_role(
        &self,
        tenant: &TenantContext,
        role_id: Uuid,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(pool.get())
            .await?;
        
        Ok(())
    }

    // Permission Management Repository Methods

    /// Lists all permissions in the system.
    /// Note: This method renames get_all_permissions for consistency
    pub async fn list_permissions(&self, tenant: &TenantContext) -> Result<Vec<Permission>> {
        // Call the existing get_all_permissions method
        self.get_all_permissions(tenant).await
    }

    /// Gets a permission by ID.
    pub async fn get_permission_by_id(
        &self,
        tenant: &TenantContext,
        permission_id: Uuid,
    ) -> Result<Option<Permission>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let permission = sqlx::query_as::<_, Permission>(
            "SELECT * FROM permissions WHERE id = $1"
        )
        .bind(permission_id)
        .fetch_optional(pool.get())
        .await?;
        
        Ok(permission)
    }

    /// Assigns a permission to a role.
    pub async fn assign_permission_to_role(
        &self,
        tenant: &TenantContext,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<()> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        // Use INSERT ... ON CONFLICT to avoid duplicate key errors
        sqlx::query(
            "INSERT INTO role_permissions (role_id, permission_id) 
             VALUES ($1, $2) 
             ON CONFLICT (role_id, permission_id) DO NOTHING"
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(pool.get())
        .await?;
        
        Ok(())
    }

    // 2FA Management Repository Methods

    /// Gets user's 2FA status and encrypted secret.
    pub async fn get_user_2fa_status(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<Option<(String, chrono::DateTime<chrono::Utc>)>> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let result = sqlx::query!(
            "SELECT two_factor_secret_encrypted, two_factor_enabled_at 
             FROM users WHERE id = $1",
            user_id
        )
        .fetch_optional(pool.get())
        .await?;

        if let Some(row) = result {
            if let (Some(secret), Some(enabled_at)) = (row.two_factor_secret_encrypted, row.two_factor_enabled_at) {
                return Ok(Some((secret, enabled_at)));
            }
        }

        Ok(None)
    }

    /// Checks if user has 2FA enabled.
    pub async fn is_2fa_enabled(
        &self,
        tenant: &TenantContext,
        user_id: Uuid,
    ) -> Result<bool> {
        let pool = self.db.get_tenant_pool(tenant).await?;
        
        let result = sqlx::query_scalar!(
            "SELECT two_factor_enabled_at IS NOT NULL FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(pool.get())
        .await?;

        Ok(result.unwrap_or(false))
    }
}

// Type alias for workflow compatibility
pub type UserRepository = AuthRepository;
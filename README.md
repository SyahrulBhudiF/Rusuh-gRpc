# Rusuh-gRpc

This what i learn, a simple gRpc implementation using rust with tonic crates

> ill keep developing this project until i how to use gRpc with rust and develop with rust

## Project Overview

This project implements a gRPC service in Rust using the Tonic framework. It demonstrates a robust application structure following Hexagonal Architecture (also known as Ports & Adapters) principles, with a clear separation of concerns. Key features include JWT-based authentication, OTP support, PostgreSQL integration via SQLx, Redis caching, and email services.

## Folder Structure

The project is organized to reflect a Hexagonal Architecture:

-   **`proto/`**: Contains the Protocol Buffer (`.proto`) files defining the gRPC services and messages. These define the contracts for some of the "driving" adapters in the `interface` layer.
-   **`src/`**: Main source code for the application.
    -   **`application/`**: Represents the application core or use cases. It orchestrates interactions between the domain and the ports. (e.g., [`auth_use_case.rs`](src/application/auth_use_case.rs:0)). This is inside the "hexagon".
    -   **`config/`**: Manages application configuration, including database connections ([`db.rs`](src/config/db.rs:0)), environment variables ([`env.rs`](src/config/env.rs:0)), and Redis setup ([`redis.rs`](src/config/redis.rs:0)). This supports the adapters.
    -   **`core/`**: Contains core server components, such as gRPC server initialization ([`server.rs`](src/core/server.rs:0)). This is part of the driving adapter setup.
    -   **`domain/`**: The innermost part of the "hexagon," containing pure business logic, entities, and port definitions.
        -   **`entity/`**: Defines the core business data structures (e.g., [`user.rs`](src/domain/entity/user.rs:0)).
        -   **`port/`**: Declares the "Ports" – interfaces that the domain and application layers use to interact with the outside world (e.g., for database access [`db_port.rs`](src/domain/port/db_port.rs:0) or caching [`redis_port.rs`](src/domain/port/redis_port.rs:0)). These ports are implemented by "driven adapters" in the `infrastructure` layer.
        -   **`service/`**: Contains domain-specific services that encapsulate business rules (e.g., [`jwt_service.rs`](src/domain/service/jwt_service.rs:0), [`totp_service.rs`](src/domain/service/totp_service.rs:0)).
    -   **`infrastructure/`**: Contains the "Adapters" that implement the ports defined in the `domain` layer. These are the "driven adapters" that connect to external systems.
        -   **`db/`**: Database adapters (e.g., [`user_adapter.rs`](src/infrastructure/db/user_adapter.rs:0)).
        -   **`notification/`**: Adapters for external notification services (e.g., [`email_adapter.rs`](src/infrastructure/notification/email_adapter.rs:0)).
        -   **`redis/`**: Redis adapters (e.g., [`redis_adapter.rs`](src/infrastructure/redis/redis_adapter.rs:0)).
    -   **`interface/`**: Represents the "driving adapters" or the primary ways the external world interacts with the application.
        -   **`common/`**: Shared utilities for the interface layer (e.g., [`client_info.rs`](src/interface/common/client_info.rs:0)).
        -   **`grpc/`**: gRPC service handlers (driving adapters) that translate incoming requests and call application use cases (e.g., [`auth_handler.rs`](src/interface/grpc/auth_handler.rs:0)).
        -   **`interceptor/`**: gRPC interceptors, which are also part of the driving adapter mechanism (e.g., [`auth_interceptor.rs`](src/interface/grpc/interceptor/auth_interceptor.rs:0)).
    -   **`pb/`**: Auto-generated Rust code from the `.proto` files (e.g., [`auth.rs`](src/pb/auth.rs:0)). These support the gRPC driving adapters.
    -   **`util/`**: General utility modules.
        -   **`template/`**: HTML templates (e.g., [`otp.html`](src/util/template/otp.html:0)).
-   **`migrations/`**: Contains database migration files managed by SQLx.

**Other notable files at the root level:**
-   **`.env.example`**: Example environment file.
-   **`.gitignore`**: Specifies intentionally untracked files.
-   **`build.rs`**: Build script for Cargo (e.g., compiling protos).
-   **`Cargo.toml`**: Project manifest file.
-   **`Cargo.lock`**: Records exact versions of dependencies.
-   **`descriptor.bin`**: gRPC reflection descriptor file (if used).

## Architecture: Hexagonal Architecture (Ports & Adapters)

This project is structured following the **Hexagonal Architecture**, also known as the **Ports & Adapters** pattern. This architectural style aims to create loosely coupled application components that can be easily connected to their software environment, promoting testability, maintainability, and flexibility by isolating the core application logic from external concerns.

**Core Concepts:**

*   **The Hexagon (Application Core)**:
    *   At the center of the architecture lies the application core, which includes the **`domain/`** and **`application/`** layers.
    *   **`domain/`**: Contains the pure business logic, entities (data structures like [`user.rs`](src/domain/entity/user.rs:0)), and domain services (like [`jwt_service.rs`](src/domain/service/jwt_service.rs:0)). This layer is completely independent of any external technology or framework.
    *   **`application/`**: Contains the application-specific use cases (e.g., [`auth_use_case.rs`](src/application/auth_use_case.rs:0)) that orchestrate the domain logic to achieve business goals. It defines how the outside world interacts with the domain.

*   **Ports**:
    *   Defined in **`domain/port/`** (e.g., [`db_port.rs`](src/domain/port/db_port.rs:0), [`redis_port.rs`](src/domain/port/redis_port.rs:0)).
    *   Ports are interfaces that define a contract for communication between the hexagon (application core) and the outside world.
    *   There are two types of ports:
        *   **Driving Ports (or Use Case Interfaces)**: Define how external actors (like UI, test scripts, other applications) can interact with the application core. These are often implicitly defined by the public API of the `application` layer's use cases.
        *   **Driven Ports (or Service Provider Interfaces)**: Define how the application core interacts with external services or infrastructure (like databases, message queues, external APIs).

*   **Adapters**:
    *   Adapters bridge the gap between the ports and the external world. They translate requests and responses between the application core's defined ports and the specific technology or tool being used.
    *   **Driving Adapters**:
        *   Located in the **`interface/`** layer (e.g., gRPC handlers in [`interface/grpc/auth_handler.rs`](src/interface/grpc/auth_handler.rs:0), interceptors in [`interface/interceptor/auth_interceptor.rs`](src/interface/grpc/interceptor/auth_interceptor.rs:0)).
        *   These adapters initiate interaction with the application core. They adapt external requests (e.g., an HTTP request, a gRPC call, a CLI command) to calls on the application's driving ports (use cases).
    *   **Driven Adapters**:
        *   Located in the **`infrastructure/`** layer (e.g., database implementations in [`infrastructure/db/user_adapter.rs`](src/infrastructure/db/user_adapter.rs:0), Redis cache in [`infrastructure/redis/redis_adapter.rs`](src/infrastructure/redis/redis_adapter.rs:0)).
        *   These adapters are implementations of the driven ports defined by the application core. They adapt the application's calls to specific external services or tools.

**Key Principles & Benefits:**

*   **Dependency Inversion**: The application core (hexagon) defines the ports (interfaces) it needs. Adapters in the outer layers implement these ports. This means dependencies flow inwards – the core does not depend on specific external technologies.
*   **Isolation of Business Logic**: The core business logic within the `domain` and `application` layers is isolated from external concerns like UI, databases, or frameworks.
*   **Testability**: The application core can be tested in isolation by using mock or stub adapters for the ports. Driving adapters can also be tested independently.
*   **Maintainability & Flexibility**: External technologies or frameworks can be changed or replaced by simply writing a new adapter, without affecting the core application logic. For example, switching from PostgreSQL to MySQL would only require a new database adapter in the `infrastructure` layer.
*   **Technology Agnostic Core**: The hexagon remains unaware of the specific technologies used by the adapters.

This structure ensures that your application is robust, adaptable to changing requirements, and easier to maintain over time.

use super::super::client::SendClientRPC;
use super::super::common_rpc_types::{GraphCommonArgs, NodeName, ShellStartCodeChainRequest, UpdateCodeChainRequest};
use super::super::router::Router;
use super::super::rpc::{response, RPCError, RPCResponse};
use super::types::{
    Context, DashboardGetNetworkResponse, DashboardNode, GraphNetworkOutAllAVGResponse, GraphNetworkOutAllResponse,
    GraphNetworkOutNodeExtensionResponse, GraphNetworkOutNodePeerResponse, LogGetRequest, LogGetResponse,
    LogGetTargetsResponse, NodeConnection, NodeGetInfoResponse,
};

pub fn add_routing(router: &mut Router<Context>) {
    router.add_route("ping", Box::new(ping as fn(Context) -> RPCResponse<String>));
    router.add_route(
        "node_getInfo",
        Box::new(node_get_info as fn(Context, (String,)) -> RPCResponse<NodeGetInfoResponse>),
    );
    router.add_route(
        "dashboard_getNetwork",
        Box::new(dashboard_get_network as fn(Context) -> RPCResponse<DashboardGetNetworkResponse>),
    );
    router.add_route(
        "node_start",
        Box::new(node_start as fn(Context, (String, ShellStartCodeChainRequest)) -> RPCResponse<()>),
    );
    router.add_route("node_stop", Box::new(node_stop as fn(Context, (String,)) -> RPCResponse<()>));
    router.add_route(
        "node_update",
        Box::new(node_update as fn(Context, (NodeName, UpdateCodeChainRequest)) -> RPCResponse<()>),
    );
    router.add_route("log_getTargets", Box::new(log_get_targets as fn(Context) -> RPCResponse<LogGetTargetsResponse>));
    router.add_route("log_get", Box::new(log_get as fn(Context, (LogGetRequest,)) -> RPCResponse<LogGetResponse>));
    router.add_route(
        "graph_network_out_all_node",
        Box::new(
            graph_network_out_all_node as fn(Context, (GraphCommonArgs,)) -> RPCResponse<GraphNetworkOutAllResponse>,
        ),
    );
    router.add_route(
        "graph_network_out_all_node_avg",
        Box::new(
            graph_network_out_all_node_avg
                as fn(Context, (GraphCommonArgs,)) -> RPCResponse<GraphNetworkOutAllAVGResponse>,
        ),
    );
    router.add_route(
        "graph_network_out_node_extension",
        Box::new(
            graph_network_out_node_extension
                as fn(Context, (NodeName, GraphCommonArgs)) -> RPCResponse<GraphNetworkOutNodeExtensionResponse>,
        ),
    );
    router.add_route(
        "graph_network_out_node_peer",
        Box::new(
            graph_network_out_node_peer
                as fn(Context, (NodeName, GraphCommonArgs)) -> RPCResponse<GraphNetworkOutNodePeerResponse>,
        ),
    );
}

fn ping(_: Context) -> RPCResponse<String> {
    response("pong".to_string())
}

fn dashboard_get_network(context: Context) -> RPCResponse<DashboardGetNetworkResponse> {
    let clients_state = context.db_service.get_clients_state()?;
    let connections = context.db_service.get_connections()?;
    let dashboard_nodes = clients_state.iter().map(|client| DashboardNode::from_db_state(client)).collect();
    response(DashboardGetNetworkResponse {
        nodes: dashboard_nodes,
        connections: connections.iter().map(|connection| NodeConnection::from_connection(connection)).collect(),
    })
}

fn node_get_info(context: Context, args: (String,)) -> RPCResponse<NodeGetInfoResponse> {
    let (name,) = args;
    let client_query_result = context.db_service.get_client_query_result(&name)?.ok_or(RPCError::ClientNotFound)?;
    let extra = context.db_service.get_client_extra(name)?;
    response(NodeGetInfoResponse::from_db_state(&client_query_result, &extra))
}

fn node_start(context: Context, args: (NodeName, ShellStartCodeChainRequest)) -> RPCResponse<()> {
    let (name, req) = args;

    let client = context.client_service.get_client(&name);
    if client.is_none() {
        return Err(RPCError::ClientNotFound)
    }
    let client = client.expect("Already checked");
    client.shell_start_codechain(req.clone())?;

    context.db_service.save_start_option(name, &req.env, &req.args);

    response(())
}

fn node_stop(context: Context, args: (String,)) -> RPCResponse<()> {
    let (name,) = args;

    let client = context.client_service.get_client(&name);
    if client.is_none() {
        return Err(RPCError::ClientNotFound)
    }
    let client = client.expect("Already checked");
    client.shell_stop_codechain()?;

    response(())
}

fn node_update(context: Context, args: (NodeName, UpdateCodeChainRequest)) -> RPCResponse<()> {
    let (name, req) = args;

    let client = context.client_service.get_client(&name).ok_or(RPCError::ClientNotFound)?;

    let extra = context.db_service.get_client_extra(name)?;
    let (env, args) = extra.map(|extra| (extra.prev_env, extra.prev_args)).unwrap_or_default();
    client.shell_update_codechain((
        ShellStartCodeChainRequest {
            env,
            args,
        },
        req,
    ))?;

    response(())
}

fn log_get_targets(context: Context) -> RPCResponse<LogGetTargetsResponse> {
    let targets = context.db_service.get_log_targets()?;
    response(LogGetTargetsResponse {
        targets,
    })
}

fn log_get(context: Context, args: (LogGetRequest,)) -> RPCResponse<LogGetResponse> {
    let (req,) = args;
    let logs = context.db_service.get_logs(req)?;
    response(LogGetResponse {
        logs,
    })
}

fn graph_network_out_all_node(context: Context, args: (GraphCommonArgs,)) -> RPCResponse<GraphNetworkOutAllResponse> {
    let (graph_args,) = args;

    let rows = context.db_service.get_network_out_all_graph(graph_args)?;
    response(GraphNetworkOutAllResponse {
        rows,
    })
}

fn graph_network_out_all_node_avg(
    context: Context,
    args: (GraphCommonArgs,),
) -> RPCResponse<GraphNetworkOutAllAVGResponse> {
    let (graph_args,) = args;

    let rows = context.db_service.get_network_out_all_avg_graph(graph_args)?;
    response(GraphNetworkOutAllAVGResponse {
        rows,
    })
}

fn graph_network_out_node_extension(
    context: Context,
    args: (NodeName, GraphCommonArgs),
) -> RPCResponse<GraphNetworkOutNodeExtensionResponse> {
    let (node_name, graph_args) = args;

    let rows = context.db_service.get_network_out_node_extension_graph(node_name, graph_args)?;
    response(GraphNetworkOutNodeExtensionResponse {
        rows,
    })
}

fn graph_network_out_node_peer(
    context: Context,
    args: (NodeName, GraphCommonArgs),
) -> RPCResponse<GraphNetworkOutNodePeerResponse> {
    let (node_name, graph_args) = args;

    let rows = context.db_service.get_network_out_node_peer_graph(node_name, graph_args)?;
    response(GraphNetworkOutNodePeerResponse {
        rows,
    })
}

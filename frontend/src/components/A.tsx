import React, { ReactNode } from "react"
import { Link, withRouter, RouteComponentProps } from "react-router-dom"
import classNames from "classnames"

export const A = withRouter(
    (
        props: {
            to: string
            activeClassName?: string
            className?: string
            children?: ReactNode
        } & RouteComponentProps,
    ) => {
        const active = props.location.pathname === props.to
        return (
            <Link
                to={props.to}
                replace={active}
                className={active ? classNames(props.className, props.activeClassName) : props.className}>
                {props.children}
            </Link>
        )
    },
)

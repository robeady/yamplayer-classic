import React, { ReactNode } from "react"
import classNames from "classnames"
import { Link as RouterLink, LinkProps as RouterLinkProps, withRouter, RouteComponentProps } from "react-router-dom"
import { Link as RebassLink, LinkProps as RebassLinkProps } from "rebass"

export const Link = (props: RebassLinkProps & RouterLinkProps) => <RebassLink {...props} as={RouterLink} />

export const NavLink = withRouter(
    (
        props: {
            to: string
            activeClassName?: string
            className?: string
        } & RebassLinkProps &
            RouterLinkProps &
            RouteComponentProps,
    ) => {
        const active = props.location.pathname === props.to
        return (
            <Link
                {...props}
                replace={active}
                className={active ? classNames(props.className, props.activeClassName) : props.className}
            />
        )
    },
)

interface Children {
    children?: ReactNode
}

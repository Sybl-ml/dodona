import React from "react"
import { Navbar, Nav,} from 'react-bootstrap';
import styled from "styled-components";
import MemoLogo from '../icons/Logo.js';
import { FaAngleDown } from 'react-icons/fa';

import Toggle from "./Toggler"

const HeaderBar = styled(Navbar)`
    background: ${({ theme }) => theme.body};
    transition: all 0.25s linear;
`;

const UserIcon = styled.img`
	background-color: #bbb;
	border-radius: 50%;
	height:2.5rem;
	width:2.5rem;
	margin-left: 0.5rem;
`;

const DashHeader = ({theme, toggleTheme}) => {
        return (
		<HeaderBar collapseOnSelect sticky="top">
			<HeaderBar.Brand href="/home">
				<MemoLogo 
					theme={theme} 
				/> 
			</HeaderBar.Brand>
			<HeaderBar.Toggle aria-controls="responsive-navbar-nav" className="styled-toggle"/>
  			<HeaderBar.Collapse id="responsive-navbar-nav" className="justify-content-end">
				<Toggle theme={theme} toggleTheme={toggleTheme} />
				<Nav className="justify-content-center">
					<UserIcon src="https://www.gravatar.com/avatar/205e460b479e2e5b48aec07710c08d50" />
					<Nav.Link style={{paddingLeft:"0", paddingRight:"0"}}>
						<FaAngleDown/>
					</Nav.Link>
				</Nav>
			</HeaderBar.Collapse>
  		</HeaderBar>
        );
    };
export default DashHeader;
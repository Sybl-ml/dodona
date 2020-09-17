import React from "react"
import { Navbar, Nav,} from 'react-bootstrap';
import styled from "styled-components";
import MemoLogo from '../icons/Logo.js';
import {PrimaryButton} from './Buttons';

import Toggle from "./Toggler"

const HeaderBar = styled(Navbar)`
    background: ${({ theme }) => theme.body};
    transition: all 0.25s linear;
`;
const DashHeader = ({theme, toggleTheme}) => {
        return (
		<HeaderBar collapseOnSelect expand="lg" sticky="top">
			<HeaderBar.Brand href="/home">
				<MemoLogo 
					theme={theme} 
				/> 
			</HeaderBar.Brand>
			<HeaderBar.Toggle aria-controls="responsive-navbar-nav" />
  			<HeaderBar.Collapse id="responsive-navbar-nav" className="justify-content-end">
				<Toggle theme={theme} toggleTheme={toggleTheme} />
				<Nav>
					<Nav.Link href="/login">Sign In</Nav.Link>
				</Nav>
				<PrimaryButton variant="primary" href="#">Hey, John</PrimaryButton>
			</HeaderBar.Collapse>
  		</HeaderBar>
        );
    };
export default DashHeader;